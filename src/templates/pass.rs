use super::parser::{PFileEntry, PassItems};
use crate::*;
use config::*;
use err::*;
use files::*;
use gobble::Parser;
use gtmpl::Template;
use gtmpl_helpers::THelper;
use gtmpl_value::Number as GNum;
use gtmpl_value::Value as GVal;
use pulldown_cmark as cmark;
use std::collections::HashMap;
use std::io::Write;
use std::process::{Command, Stdio};
use toml::Value as TVal;

#[derive(Clone, PartialEq, Debug)]
pub struct Section<'a> {
    pub passes: Option<Vec<Pass>>,
    pub s: &'a str,
}

pub struct FileEntry {
    pub path: String,
    pub var: Option<String>,
}

fn passes(v: &Vec<Pass>, s: &str, data: &mut Config) -> anyhow::Result<String> {
    let mut it = v.iter();

    let mut rs = match it.next() {
        Some(p) => p.pass(s, data)?,
        None => return Ok(s.to_string()),
    };
    while let Some(pass) = it.next() {
        rs = pass.pass(&rs, data)?;
    }
    Ok(rs)
}

impl<'a> Section<'a> {
    pub fn pass(&self, data: &mut Config, fs: &FSource) -> anyhow::Result<String> {
        match &self.passes {
            Some(p) => passes(p, self.s, data),
            None => passes(&fs.def_pass(data)?, self.s, data),
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum FSource {
    Content,
    Templates,
    Static,
}

impl FSource {
    pub fn str(&self) -> &'static str {
        match self {
            FSource::Content => CONTENT,
            FSource::Templates => TEMPLATES,
            FSource::Static => STATIC,
        }
    }
    pub fn def_pass_str(&self) -> &'static str {
        match self {
            FSource::Content => CONTENT_DEF_PASS,
            FSource::Templates => TEMPLATES_DEF_PASS,
            FSource::Static => STATIC_DEF_PASS,
        }
    }

    pub fn def_pass(&self, cf: &dyn Configger) -> anyhow::Result<Vec<Pass>> {
        let p_str = match cf.get_str(self.def_pass_str()) {
            Some(p) => p,
            None => return Ok(Vec::new()),
        };
        PassItems
            .parse_s(p_str)
            .map_err(|e| e.strung(p_str.to_string()).into())
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum Pass {
    Comment,
    Toml,
    GTemplate,
    Markdown,
    Files(FSource),
    Dirs(FSource),
    Template(Option<String>),
    Set(String),
    Table(String),
    Exec(String),
}

impl Pass {
    fn pass(&self, s: &str, data: &mut Config) -> anyhow::Result<String> {
        match self {
            Pass::Toml => {
                let t: TVal = s.parse()?;
                for (k, v) in toml_util::as_table(&t)? {
                    data.t_insert(k, v.clone());
                }
                Ok(String::new())
            }
            Pass::Files(sc) => {
                let mut res = String::new();
                for f in s
                    .split("\n")
                    .filter_map(|s| PFileEntry.parse_s(s).ok())
                    .filter(|e| e.path.len() > 0)
                {
                    let r = load_locale(&f.path, sc.str(), data)?;
                    match f.var {
                        Some(v) => data.t_insert(v, r),
                        None => res.push_str(&r),
                    }
                }
                Ok(res)
            }
            Pass::Dirs(sc) => {
                let mut res = String::new();
                for f in s
                    .split("\n")
                    .filter_map(|s| PFileEntry.parse_s(s).ok())
                    .filter(|e| e.path.len() > 0)
                {
                    let r = read_locale_dir(&f.path, sc.str(), data)?;
                    match f.var {
                        Some(v) => data.t_insert(v, r),
                        None => {
                            for x in r {
                                res.push_str(&x.to_string());
                            }
                        }
                    }
                }
                Ok(res)
            }

            Pass::Template(nm) => match nm {
                None => super::run_mut(data, s, &FSource::Templates),
                Some(nm) => {
                    let part = load_locale(nm, TEMPLATES, &data)?;
                    let res = super::run_mut(data, &part, &FSource::Templates);
                    res
                }
            },
            Pass::Markdown => {
                let p = cmark::Parser::new(s);
                let mut res = String::new();
                cmark::html::push_html(&mut res, p);
                Ok(res)
            }
            Pass::GTemplate => {
                let gdat = data.to_gtmpl();

                let mut tp = Template::default().with_defaults();
                //Add other useful features

                tp.parse(s).map_err(|e| err(e))?;
                tp.q_render(gdat).map_err(|e| err(e).into())
            }
            Pass::Set(v) => {
                data.t_insert(v, s.to_string());
                Ok(String::new())
            }
            Pass::Table(istr) => Ok(format!(
                "<table {}>\n{}</table>\n",
                istr,
                super::table::Table
                    .parse_s(s)
                    .map_err(|e| e.strung(s.to_string()))?
            )),
            Pass::Exec(estr) => {
                let mut sp = estr.split(" ").filter(|&x| x != ""); // TODO parse properly
                let mut p = Command::new(sp.next().ok_or(Error::NoExecCommand)?)
                    .args(sp)
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .spawn()?;
                if let Some(ref mut w) = p.stdin {
                    w.write_all(s.as_bytes())?;
                }
                let op = p.wait_with_output()?;
                Ok(String::from_utf8(op.stdout.as_slice().to_vec())?)
            }
            Pass::Comment => Ok(String::new()),
        }
    }
}

pub fn map_to_gtmpl(tb: &TMap) -> GVal {
    let mut m = HashMap::new();
    for (k, v) in tb {
        m.insert(k.to_string(), toml_to_gtmpl(v));
    }
    GVal::Map(m)
}

fn table_to_gtmpl(tb: &toml::value::Table) -> GVal {
    let mut m = HashMap::new();
    for (k, v) in tb {
        m.insert(k.to_string(), toml_to_gtmpl(v));
    }
    GVal::Map(m)
}
pub fn toml_to_gtmpl(t: &TVal) -> GVal {
    match t {
        TVal::String(s) => GVal::String(s.clone()),
        TVal::Integer(i) => GVal::Number(GNum::from(*i)),
        TVal::Float(f) => GVal::Number(GNum::from(*f)),
        TVal::Boolean(b) => GVal::Bool(*b),
        TVal::Datetime(dt) => GVal::String(dt.to_string()),
        TVal::Array(a) => GVal::Array(a.into_iter().map(|t| toml_to_gtmpl(t)).collect()),
        TVal::Table(tb) => table_to_gtmpl(tb),
    }
}

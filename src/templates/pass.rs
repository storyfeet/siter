use crate::err::Error as SiteErr;
use crate::toml_util;
use gobble::*;
use gtmpl::Template;
use gtmpl_helpers::THelper;
use gtmpl_value::Number as GNum;
use gtmpl_value::Value as GVal;
use pulldown_cmark as cmark;
use std::collections::HashMap;
use std::io::Write;
use std::process::{Command, Stdio};
//use toml::value::Table;
use crate::config::CMap;
use toml::Value as TVal;

#[derive(Clone, PartialEq, Debug)]
pub struct Section<'a> {
    pub passes: Vec<Pass>,
    pub s: &'a str,
}

impl<'a> Section<'a> {
    pub fn pass(&self, data: &mut CMap) -> anyhow::Result<String> {
        let mut it = self.passes.iter();
        let mut rs = it.next().unwrap_or(&Pass::None).pass(self.s, data)?;
        while let Some(pass) = it.next() {
            rs = pass.pass(&rs, data)?;
        }
        Ok(rs)
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum Pass {
    None,
    Comment,
    Toml,
    GTemplate,
    Markdown,
    Table(String),
    Exec(String),
}

impl Pass {
    fn pass(&self, s: &str, data: &mut CMap) -> anyhow::Result<String> {
        match self {
            Pass::None => Ok(s.to_string()),
            Pass::Toml => {
                let t: TVal = s.parse()?;
                for (k, v) in toml_util::as_table(&t)? {
                    data.insert(k.clone(), v.clone());
                }
                Ok(String::new())
            }
            Pass::Markdown => {
                let p = cmark::Parser::new(s);
                let mut res = String::new();
                cmark::html::push_html(&mut res, p);
                Ok(res)
            }
            Pass::GTemplate => {
                let gdat = map_to_gtmpl(data);
                let mut tp = Template::default().with_defaults();
                tp.parse(s).map_err(|e| SiteErr::String(e))?;
                tp.q_render(gdat).map_err(|e| SiteErr::String(e).into())
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
                let mut p = Command::new(sp.next().ok_or(SiteErr::NoExecCommand)?)
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

pub fn map_to_gtmpl(tb: &CMap) -> GVal {
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

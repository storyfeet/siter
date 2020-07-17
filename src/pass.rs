use gobble::*;
use gtmpl::Template;
use gtmpl_helpers::THelper;
use gtmpl_value::Number as GNum;
use gtmpl_value::Value as GVal;
use pulldown_cmark as cmark;
use std::collections::HashMap;
use toml::value::Table;
use toml::Value as TVal;

#[derive(Clone, PartialEq, Debug)]
pub struct Section<'a> {
    pub passes: Vec<Pass>,
    pub s: &'a str,
}

impl<'a> Section<'a> {
    pub fn pass(&self, data: &mut Table) -> anyhow::Result<String> {
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
    Toml,
    GTemplate,
    Markdown,
    Table(String),
    Exec(String),
}

impl Pass {
    fn pass(&self, s: &str, data: &mut Table) -> anyhow::Result<String> {
        match self {
            Pass::None => Ok(s.to_string()),
            Pass::Toml => {
                let t: TVal = s.parse()?;
                match t {
                    TVal::Table(tb) => {
                        for (k, v) in tb {
                            data.insert(k, v);
                        }
                    }
                    _ => return Err(crate::err::Error::TomlNotMap.into()),
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
                let gdat = table_to_gtmpl(data);
                let mut tp = Template::default().with_defaults();
                tp.parse(s).map_err(|e| crate::err::Error::String(e))?;
                tp.q_render(gdat)
                    .map_err(|e| crate::err::Error::String(e).into())
            }
            Pass::Table(istr) => Ok(format!(
                "<table {}>\n{}</table>\n",
                istr,
                crate::table::Table
                    .parse_s(s)
                    .map_err(|e| e.strung(s.to_string()))?
            )),
            _ => Ok(String::new()),
        }
    }
}

fn table_to_gtmpl(tb: &Table) -> GVal {
    let mut m = HashMap::new();
    for (k, v) in tb {
        m.insert(k.to_string(), toml_to_gtmpl(v));
    }
    GVal::Map(m)
}

fn toml_to_gtmpl(t: &TVal) -> GVal {
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

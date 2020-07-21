pub mod parser;
pub mod pass;
pub mod table;
use crate::config::Config;
use crate::err::*;
use crate::util;
use gobble::err::StrungError;
use std::fmt::Write;
use std::path::PathBuf;
use std::rc::Rc;

pub fn run(conf: Rc<Config>, s: &str) -> anyhow::Result<Rc<Config>> {
    let mut res_str = String::new();
    let p = parser::section_pull(s);
    let mut conf = Config::new().parent(conf.clone());
    for set_res in p {
        let set = set_res.map_err(|e| StrungError::from(e))?;
        let pd = &set
            .pass(&mut conf)
            .map_err(|e| EWrap::new(set.s.to_string(), e))?;
        if pd != "" {
            writeln!(res_str, "{}", pd)?;
        }
    }
    conf.insert("result", res_str);
    Ok(Rc::new(conf))
}

pub fn run_to<W: std::io::Write>(
    w: &mut W,
    conf: Rc<Config>,
    s: &str,
) -> anyhow::Result<Rc<Config>> {
    let p = parser::section_pull(s);
    let mut conf = Config::new().parent(conf.clone());
    for set_res in p {
        let set = set_res.map_err(|e| StrungError::from(e))?;
        let pd = &set
            .pass(&mut conf)
            .map_err(|e| EWrap::new(set.s.to_string(), e))?;
        if pd != "" {
            writeln!(w, "{}", pd)?;
        }
    }
    Ok(Rc::new(conf))
}

pub fn load_template(conf: &Config) -> anyhow::Result<String> {
    let name = conf.get_str("type").unwrap_or("page");
    for c in conf
        .get_strs("templates")
        .ok_or(s_err("Template folder provided"))?
    {
        let fp = PathBuf::from(c).join(name);
        match util::read_file(fp) {
            Ok(f) => Some(f),
            Err(_) => continue,
        };
    }
    Err(Error::String(format!("No template for type '{}'", name)).into())
}

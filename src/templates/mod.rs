pub mod parser;
pub mod pass;
pub mod table;
use crate::err::*;
use crate::util;
use gobble::err::StrungError;
//use toml::value::Table;
use crate::config::{CMap, Config};
use std::path::PathBuf;

pub fn get_data(s: &str) -> anyhow::Result<CMap> {
    let p = parser::section_pull(s);

    let mut dt = CMap::new();
    for set_res in p {
        let set = set_res.map_err(|e| StrungError::from(e))?;
        set.pass(&mut dt)
            .map_err(|e| EWrap::new(set.s.to_string(), e))?;
    }
    Ok(dt)
}

pub fn run_to<W: std::fmt::Write>(w: &mut W, s: &str) -> anyhow::Result<CMap> {
    let p = parser::section_pull(s);
    let mut dt = CMap::new();
    for set_res in p {
        let set = set_res.map_err(|e| StrungError::from(e))?;
        let pd = &set
            .pass(&mut dt)
            .map_err(|e| EWrap::new(set.s.to_string(), e))?;
        if pd != "" {
            writeln!(w, "{}", pd)?;
        }
    }
    Ok(dt)
}

pub fn run_to_io<W: std::io::Write>(w: &mut W, s: &str) -> anyhow::Result<CMap> {
    let p = parser::section_pull(s);
    let mut dt = CMap::new();
    for set_res in p {
        let set = set_res.map_err(|e| StrungError::from(e))?;
        let pd = &set
            .pass(&mut dt)
            .map_err(|e| EWrap::new(set.s.to_string(), e))?;
        if pd != "" {
            writeln!(w, "{}", pd)?;
        }
    }
    Ok(dt)
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

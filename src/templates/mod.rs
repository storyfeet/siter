pub mod parser;
pub mod pass;
pub mod table;
use crate::*;
use config::*;
use err::*;
use pass::FSource;

use gobble::err::StrungError;
use std::fmt::Write;

pub fn run_mut(conf: &mut Config, s: &str, source_type: &FSource) -> anyhow::Result<String> {
    let mut res_str = String::new();
    let mut p = parser::section_pull(s);
    if let Some(first) = p.next() {
        let mut set = first.map_err(|e| StrungError::from(e))?;
        set.passes = source_type.def_pass(conf)?;
        let pd = &set
            .pass(conf)
            .map_err(|e| EWrap::new(set.s.to_string(), e))?;
        if pd != "" {
            writeln!(res_str, "{}", pd)?;
        }
    }
    for set_res in p {
        let set = set_res.map_err(|e| StrungError::from(e))?;
        let pd = &set
            .pass(conf)
            .map_err(|e| EWrap::new(set.s.to_string(), e))?;
        if pd != "" {
            writeln!(res_str, "{}", pd)?;
        }
    }
    Ok(res_str)
}

pub fn run<'a>(
    conf: &'a dyn Configger,
    s: &str,
    stype: &FSource,
) -> anyhow::Result<(String, Config<'a>)> {
    let mut rconf = Config::new(conf);
    let rstr = run_mut(&mut rconf, s, stype)?;
    Ok((rstr, rconf))
}

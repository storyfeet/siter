pub mod parser;
pub mod pass;
pub mod table;
use crate::*;
use config::*;
use err::*;

use gobble::err::StrungError;
use gobble::Parser;
use std::fmt::Write;

pub fn run_mut(conf: &mut Config, s: &str) -> anyhow::Result<String> {
    let mut res_str = String::new();
    let mut p = parser::section_pull(s);
    if let Some(pass_str) = conf.get_str(DEFAULT_PASS) {
        if let Ok(ps) = parser::PassItems.parse_s(&pass_str) {
            if let Some(first) = p.next() {
                let mut set = first.map_err(|e| StrungError::from(e))?;
                set.passes = ps;
                let pd = &set
                    .pass(conf)
                    .map_err(|e| EWrap::new(set.s.to_string(), e))?;
                if pd != "" {
                    writeln!(res_str, "{}", pd)?;
                }
            }
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

pub fn run<'a>(conf: &'a dyn Configger, s: &str) -> anyhow::Result<Config<'a>> {
    //println!("Running -- {:?}", conf);
    let mut conf = RootConfig::new().parent(conf.clone());
    let rs = run_mut(&mut conf, s)?;
    conf.insert("result", rs);
    Ok(conf)
}

pub fn run_to<'a, W: std::io::Write>(
    w: &mut W,
    conf: &'a dyn Configger,
    s: &str,
) -> anyhow::Result<Config<'a>> {
    //println!("\nRunning to -- {:?}\n\n", conf);
    let p = parser::section_pull(s);
    let mut conf = RootConfig::new().parent(conf.clone());
    for set_res in p {
        let set = set_res.map_err(|e| StrungError::from(e))?;
        let pd = &set
            .pass(&mut conf)
            .map_err(|e| EWrap::new(set.s.to_string(), e))?;
        if pd != "" {
            writeln!(w, "{}", pd)?;
        }
    }
    Ok(conf)
}

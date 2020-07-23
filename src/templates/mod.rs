pub mod parser;
pub mod pass;
pub mod table;
use crate::config::Config;
use crate::err::*;

use gobble::err::StrungError;
use std::fmt::Write;
use std::rc::Rc;

pub fn run_mut(conf: &mut Config, s: &str) -> anyhow::Result<String> {
    let mut res_str = String::new();
    let p = parser::section_pull(s);
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

pub fn run(conf: Rc<Config>, s: &str) -> anyhow::Result<Rc<Config>> {
    //println!("Running -- {:?}", conf);
    let mut conf = Config::new().parent(conf.clone());
    let rs = run_mut(&mut conf, s)?;
    conf.insert("result", rs);
    Ok(Rc::new(conf))
}

pub fn run_to<W: std::io::Write>(
    w: &mut W,
    conf: Rc<Config>,
    s: &str,
) -> anyhow::Result<Rc<Config>> {
    //println!("\nRunning to -- {:?}\n\n", conf);
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

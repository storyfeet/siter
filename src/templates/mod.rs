pub mod parser;
pub mod pass;
pub mod table;
use crate::err::EWrap;
use gobble::err::StrungError;
use toml::value::Table;

pub fn get_data(s: &str) -> anyhow::Result<Table> {
    let p = parser::section_pull(s);

    let mut dt = Table::new();
    for set_res in p {
        let set = set_res.map_err(|e| StrungError::from(e))?;
        set.pass(&mut dt)
            .map_err(|e| EWrap::new(set.s.to_string(), e))?;
    }
    Ok(dt)
}

pub fn run_to<W: std::io::Write>(w: &mut W, s: &str) -> anyhow::Result<Table> {
    let p = parser::section_pull(s);

    let mut dt = Table::new();
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

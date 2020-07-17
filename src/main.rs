use std::io::Read;
mod err;
mod parser;
mod pass;
mod table;
use gobble::StrungError;
use toml::value::Table;

fn main() -> anyhow::Result<()> {
    for x in std::env::args().skip(1) {
        let mut s = String::new();
        std::fs::File::open(x)
            .expect("No File")
            .read_to_string(&mut s)
            .expect("No good file");

        let p = parser::section_pull(&s);

        let mut res = String::new();

        let mut dt = Table::new();
        for set_res in p {
            let set = set_res.map_err(|e| StrungError::from(e))?;
            let pd = &set
                .pass(&mut dt)
                .map_err(|e| err::EWrap::new(set.s.to_string(), e))?;
            res.push_str(pd);
        }
        println!("{}", res);
    }
    Ok(())
}

//use gobble::StrungError;
use siter::{templates, toml_util};
use std::io::Read;
//use toml::value::Table;
use clap_conf::*;

fn main() -> anyhow::Result<()> {
    let clp = clap_app!(siter_gen =>
        (about:"A Program to generate a site from multiple forms of output")
        (version:crate_version!())
        (author:"Matthew Stoodley")
        (@arg root:-r --root +takes_value "The root of the project to work with")
        (@arg output:-o --output +takes_value "The folder to put the output default='public'")
        (@arg templates:-t --templates +takes_value ... "The list of folders to find templates in default='[templates]'")
        (@arg content:--content +takes_value ... "The list of folders where to find content default='[content]'")
        (@arg statics:-s --static +takes_value ... "The list of folders where to find static content default='[static]'")
    )
    .get_matches();

    let conf = &clp;
    //Get base Data
    let root = conf.grab_local().arg("root").def(std::env::current_dir()?);
    let mut root_conf = toml_util::load_toml(&root).unwrap_or_else(|_| {
        println!("No Root Toml Provided Working with defaults");
        toml::Value::Table(toml::value::Table::new())
    });

    //Build templates

    //build content

    //build static

    for x in std::env::args().skip(1) {
        let mut s = String::new();
        std::fs::File::open(x)?.read_to_string(&mut s)?;
        templates::run_to(&mut std::io::stdout(), &s)?;
    }
    Ok(())
}

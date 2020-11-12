use clap::ArgMatches;
use clap_conf::*;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::str::FromStr;

use err_tools::*;
use templito::prelude::*;

const SVG_DIMS: &str = "<svg></svg>";

fn get_named_template(name: &str) -> Option<&'static str> {
    match name {
        "svg_dims" => Some(SVG_DIMS),
        _ => None,
    }
}

fn get_template(conf: &ArgMatches) -> anyhow::Result<TreeTemplate> {
    if let Some(v) = conf.value_of("t_name") {
        let t = get_named_template(v).e_str("Template by that name does not exist")?;
        return TreeTemplate::from_str(t).into();
    }
    if let Some(fname) = conf.value_of("f_file") {
        let mut tf = std::fs::File::open(fname)?;
        let mut s = String::new();
        tf.read_to_string(&mut s)?;
        return TreeTemplate::from_str(&s).into();
    }

    let s_in = std::io::stdin();
    let mut s = String::new();
    s_in.lock().read_to_string(&mut s)?;
    TreeTemplate::from_str(&s).into()
}

pub fn exec(conf: &ArgMatches) -> anyhow::Result<()> {
    //Get the template
    let t = get_template(conf)?;

    //Get the data values
    let mut data = Vec::new();
    for s in conf.values_of("data")? {}

    //get the folder locks

    //Set the output target

    //run the template
    Ok(())
}

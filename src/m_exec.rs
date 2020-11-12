use clap::ArgMatches;
use clap_conf::*;
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
        let tt = TreeTemplate::from_str(t)?;
    }
}

pub fn exec(conf: &ArgMatches) {
    //Get the template

    //Get the data values

    //get the folder locks

    //Set the output target

    //run the template
}

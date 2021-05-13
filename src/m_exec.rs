use clap::ArgMatches;
use std::io::Read;
use std::str::FromStr;

use err_tools::*;
use templito::prelude::*;

const SVG_DIMS: &str = include_str!("../templates/svg_pics.ito");

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
    if let Some(fname) = conf.value_of("t_file") {
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
    //println!("template recieved");

    //Get the data values
    let mut data = Vec::new();
    if let Some(d_args) = conf.values_of("data") {
        for s in d_args {
            match TData::from_str(s) {
                Ok(v) => data.push(v),
                Err(_) => data.push(TData::String(s.to_string())),
            }
        }
    }
    let mut bdata: Vec<&dyn TParam> = Vec::new();
    for a in &data {
        bdata.push(a);
    }

    let fman = default_func_man().with_exec().with_free_files();

    let mut tman = BasicTemps::new();
    //get the folder locks
    let res = t.run(&bdata, &mut tman, &fman)?;

    print!("{}", res);

    Ok(())
}

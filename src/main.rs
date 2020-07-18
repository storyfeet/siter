//use gobble::StrungError;
use siter::{config, err, templates};
use std::io::Read;
use std::rc::Rc;
//use toml::value::Table;
use clap_conf::*;
use config::Config;
use std::path::{Path};

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

    let mut base_conf = Config::new();
    base_conf.insert("templates", vec!["templates".to_string()]);
    base_conf.insert("content", vec!["content".to_string()]);
    base_conf.insert("static", vec!["static".to_string()]);
    base_conf.insert("output", "output".to_string());
    let base_conf = Rc::new(base_conf);

    let conf = &clp;
    //Get base Data
    let root = conf.grab_local().arg("root").def(std::env::current_dir()?);
    let mut root_conf = Config::load(&root)
        .unwrap_or(Config::new())
        .parent(base_conf);

    let mut add_v = |s: &str| {
        if let Some(ct) = clp.values_of(s) {
            let ar: Vec<toml::Value> = ct.map(|v| toml::Value::String(v.to_string())).collect();
            root_conf.insert(s, ar);
        }
    };
    add_v("templates");
    add_v("content");
    add_v("static");

    if let Some(out) = clp.value_of("output") {
        root_conf.insert("output", out)
    }
    let root_conf = Rc::new(root_conf);

    //build content

    for c in root_conf
        .get_strs("content")
        .ok_or(err::s_err("Content folders not listed"))?
    {
        let rootbuf = root.clone();
        let pb = rootbuf.join(c);
        content_folder(&pb, &root, root_conf.clone())?;
    }

    //build static

    for x in std::env::args().skip(1) {
        let mut s = String::new();
        std::fs::File::open(x)?.read_to_string(&mut s)?;
        templates::run_to(&mut std::io::stdout(), &s)?;
    }
    Ok(())
}

pub fn content_folder(p: &Path, root: &Path, mut conf: Rc<Config>) -> anyhow::Result<()> {
    let cpath = p.join("config.toml");
    if let Ok(v) = Config::load(cpath) {
        let v = v.parent(conf.clone());
        conf = Rc::new(v);
    }
    for d in std::fs::read_dir(p)?.filter_map(|s| s.ok()).filter(|f| 
        match f.path().extension() {
            Some(os) => os == ".toml",
            None => true,
        } && !f.path().starts_with("_")
    ) {
        let ft = d.file_type()?
        if ft.is_dir() {
            content_folder(&d.path(),root,conf.clone())?;
        }else if ft.is_file(){
             
            templates::get_data(ft.path())?;
        }
    }

    Ok(())
}

pub fn content_file(ft.path())

//use gobble::StrungError;
use siter::{config, err, templates,util};
use std::io::Read;
use std::rc::Rc;
//use toml::value::Table;
use clap_conf::*;
use config::Config;
use std::path::{Path,PathBuf};

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
        templates::run_to_io(&mut std::io::stdout(), &s)?;
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
        let ft = d.file_type()?;
        let mut f_conf = Config::new().parent(conf.clone());
        f_conf.insert("content_path",d.path().display().to_string());
        if ft.is_dir() {
            content_folder(&d.path(),root,Rc::new(f_conf))?;
        }else if ft.is_file(){
            content_file(&p.join(d.path()),Rc::new(f_conf))?;
        }else {
            //TODO Symlink
        }
    }

    Ok(())
}

pub fn content_file(p:&Path, conf:Rc<Config>)->anyhow::Result<()>{
    let fstr = util::read_file(p)?;
    let mut r_str = String::new();
    let cdata = templates::run_to(&mut r_str,&fstr)?;
    let mut conf = Config::from_map(cdata).parent(conf.clone());
    conf.insert("content",r_str);     

    let temp = templates::load_template(&conf)?;

    //work out destination and build path
    let mut target = conf.get_built_path("content_path").ok_or(err::s_err("No Path for Content"))?;
    if target.is_absolute() {
        target = PathBuf::from(target.display().to_string().trim_start_matches("/"));
    }
    let mut out_file = conf.get_built_path("output").unwrap_or(PathBuf::from("public"));
    out_file.push(target); 
    
    if let Some(par) = out_file.parent(){
        std::fs::create_dir_all(par)?;
    }

    // run
    let f = std::fs::OpenOptions::new().write(true).create(true).open(out_file);
    templates::write_
    target_write_

    

    Ok(()) 
}




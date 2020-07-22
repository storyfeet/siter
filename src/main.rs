//use gobble::StrungError;
use siter::err::*;
use siter::{config, templates, util};

use std::rc::Rc;
//use toml::value::Table;
use clap_conf::*;
use config::Config;
use std::path::{Path, PathBuf};

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
    let root = conf
        .grab_local()
        .arg("root")
        .def(std::env::current_dir()?.join("root_config.toml"));
    let root_folder = root.parent().ok_or(s_err("no parent folder for root"))?;

    let mut root_conf = Config::new();
    root_conf.insert("templates", vec!["templates".to_string()]);
    root_conf.insert("content", vec!["content".to_string()]);
    root_conf.insert("static", vec!["static".to_string()]);
    root_conf.insert("output", "public");
    root_conf.insert("root_folder", root_folder.display().to_string());
    root_conf.insert("root_file", root.display().to_string());
    root_conf.insert("ext", "html");
    root_conf.insert("type", "page.html");
    root_conf.insert("path_list", vec!["out_path"]);

    let root_conf = Rc::new(
        Config::load(&root)
            .wrap(format!("no root file : {} ", &root.display()))?
            .parent(Rc::new(root_conf)),
    );

    //build content

    for c in root_conf
        .get_strs("content")
        .ok_or(s_err("Content folders not listed"))?
    {
        let rootbuf = root_folder.clone();
        let pb = rootbuf.join(c);
        content_folder(&pb, &root_folder, root_conf.clone())?;
    }

    //build static
    for c in root_conf
        .get_strs("static")
        .ok_or(s_err("Static folders not listed"))?
    {
        let pb = root_folder.join(c);
        println!("running static = {}", pb.display());
        static_folder(&pb, &root_folder, root_conf.clone())?;
    }

    Ok(())
}

pub fn content_folder(p: &Path, root: &Path, mut conf: Rc<Config>) -> anyhow::Result<()> {
    println!("processing content folder {}", p.display());
    let cpath = p.join("config.toml");
    if let Ok(v) = Config::load(cpath) {
        let v = v.parent(conf.clone());
        conf = Rc::new(v);
    }
    for d in std::fs::read_dir(p)
        .wrap(format!("{}", p.display()))?
        .filter_map(|s| s.ok())
        .filter(|f| {
            !util::file_name(&f.path()).unwrap_or("_").starts_with("_")
                && match f.path().extension() {
                    Some(os) => os != ".toml",
                    None => true,
                }
        })
    {
        println!("processing folder entry {:?}", d);
        let ft = d.file_type()?;
        let mut f_conf = Config::new().parent(conf.clone());
        f_conf.insert(
            "out_path",
            util::file_name(&d.path()).ok_or(s_err("File name no worky"))?,
        );
        if ft.is_dir() {
            content_folder(&d.path(), root, Rc::new(f_conf))?;
        } else if ft.is_file() {
            content_file(&d.path(), root, Rc::new(f_conf))?;
        } else {
            //TODO Symlink
        }
    }

    Ok(())
}

pub fn content_file(p: &Path, root: &Path, conf: Rc<Config>) -> anyhow::Result<()> {
    let fstr = util::read_file(p)?;
    let conf = templates::run(conf, &fstr)?;

    let temp = templates::load_template(&conf)?;

    //work out destination and build path
    //
    let mut l_target = get_out_path(root, &conf)?;
    let ext = conf.get_str("ext").unwrap_or("html");
    l_target = l_target.with_extension(ext);
    println!("Outputting : {}", l_target.display());

    if let Some(par) = l_target.parent() {
        std::fs::create_dir_all(par)?;
    }

    // run
    let mut f = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open(l_target)?;
    templates::run_to(&mut f, conf, &temp)?;

    Ok(())
}

pub fn static_folder(p: &Path, root: &Path, mut conf: Rc<Config>) -> anyhow::Result<()> {
    let cpath = p.join("config.toml");
    if let Ok(v) = Config::load(cpath) {
        let v = v.parent(conf.clone());
        conf = Rc::new(v);
    }

    for d in std::fs::read_dir(p)
        .wrap(format!("{}", p.display()))?
        .filter_map(|s| s.ok())
    {
        println!("processing static folder entry {:?}", d);
        let ft = d.file_type()?;
        let mut f_conf = Config::new().parent(conf.clone());
        f_conf.insert(
            "out_path",
            util::file_name(&d.path()).ok_or(s_err("File name no worky"))?,
        );
        if ft.is_dir() {
            static_folder(&d.path(), root, Rc::new(f_conf))?;
        } else if ft.is_file() {
            let out_path = get_out_path(root, &f_conf)?;

            if let Some(par) = out_path.parent() {
                std::fs::create_dir_all(par)?;
            }
            std::fs::copy(d.path(), &out_path)?;
        //content_file(&d.path(), root, Rc::new(f_conf))?;
        } else {
            //TODO Symlink
        }
    }

    Ok(())
}

pub fn get_out_path(root: &Path, conf: &Config) -> anyhow::Result<PathBuf> {
    let mut target = conf
        .get_built_path("out_path")
        .ok_or(s_err("No Path for Content"))?;
    if target.is_absolute() {
        target = PathBuf::from(target.display().to_string().trim_start_matches("/"));
    }
    let out_file = conf.get_str("output").unwrap_or("public");

    let mut l_target = root.join(out_file);
    l_target.push(target);
    Ok(l_target)
}

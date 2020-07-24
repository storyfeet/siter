//use gobble::StrungError;
use files::*;
use siter::*;
use siter::{err::*, templates::pass::*};

//use toml::value::Table;
use clap_conf::*;
use config::*;
use std::io::Write;
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

    let mut root_conf = RootConfig::new();
    root_conf.t_insert("templates", vec!["templates".to_string()]);
    root_conf.t_insert("content", vec!["content".to_string()]);
    root_conf.t_insert("static", vec!["static".to_string()]);
    root_conf.t_insert("output", "public");
    root_conf.t_insert("root_folder", root_folder.display().to_string());
    root_conf.t_insert("root_file", root.display().to_string());
    root_conf.t_insert("ext", "html");
    root_conf.t_insert("type", "page.html");
    root_conf.t_insert("path_list", vec!["out_path"]);

    let root_conf = RootConfig::load(&root)
        .wrap(format!("no root file : {} ", &root.display()))?
        .parent(&root_conf);

    //build content

    for c in root_conf
        .get_strs("content")
        .ok_or(s_err("Content folders not listed"))?
    {
        let rootbuf = root_folder.clone();
        let pb = rootbuf.join(&c);
        let mut rc = RootConfig::new().parent(&root_conf);
        rc.t_insert(CONTENT_FOLDER, c);
        rc.t_insert(CONTENT_FOLDER_PATH, pb.display().to_string());

        content_folder(&pb, &root_folder, &rc)?;
    }

    //build static
    for c in root_conf
        .get_strs("static")
        .ok_or(s_err("Static folders not listed"))?
    {
        let pb = root_folder.join(c);
        println!("running static = {}", pb.display());
        static_folder(&pb, &root_folder, &root_conf)?;
    }

    Ok(())
}

pub fn content_folder(p: &Path, root: &Path, conf: &dyn Configger) -> anyhow::Result<()> {
    println!("processing content folder {}", p.display());
    let cpath = p.join("config.toml");
    let conf = match RootConfig::load(cpath) {
        Ok(v) => v.parent(conf),
        Err(_) => RootConfig::new().parent(conf),
    };
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
        let mut f_conf = RootConfig::new().parent(&conf);
        f_conf.t_insert(
            "out_path",
            util::file_name(&d.path()).ok_or(s_err("File name no worky"))?,
        );
        if ft.is_dir() {
            content_folder(&d.path(), root, &f_conf)?;
        } else if ft.is_file() {
            content_file(&d.path(), root, &f_conf)?;
        } else {
            //TODO Symlink
        }
    }

    Ok(())
}

pub fn content_file(p: &Path, root: &Path, conf: &dyn Configger) -> anyhow::Result<()> {
    let fstr = read_file(p)?;
    let (r_str, mut conf) =
        templates::run(conf, &fstr, &FSource::Content).wrap(format!("At path {}", p.display()))?;

    conf.t_insert("result", r_str);

    let temp = load_template(&conf)?;

    // run
    let (out_str, out_conf) = templates::run(&conf, &temp, &FSource::Templates)
        .wrap(format!("On file {}", p.display()))?;

    //Work out ouput file details and write

    let mut l_target = get_out_path(root, &out_conf)?;
    let ext = conf.get_str("ext").unwrap_or("html");
    l_target = l_target.with_extension(ext);
    if let Some(par) = l_target.parent() {
        std::fs::create_dir_all(par)?;
    }
    println!("Outputting : {}", l_target.display());
    let mut f = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open(&l_target)?;
    write!(f, "{}", out_str).wrap(format!("Could not write {}", l_target.display()))?;
    Ok(())
}

pub fn static_folder(p: &Path, root: &Path, conf: &Config) -> anyhow::Result<()> {
    let cpath = p.join("config.toml");
    let conf = match RootConfig::load(cpath) {
        Ok(v) => v.parent(conf),
        Err(_) => RootConfig::new().parent(conf),
    };

    for d in std::fs::read_dir(p)
        .wrap(format!("{}", p.display()))?
        .filter_map(|s| s.ok())
    {
        println!("processing static folder entry {:?}", d);
        let ft = d.file_type()?;
        let mut f_conf = Config::new(&conf);
        f_conf.t_insert(
            "out_path",
            util::file_name(&d.path()).ok_or(s_err("File name no worky"))?,
        );
        if ft.is_dir() {
            static_folder(&d.path(), root, &f_conf)?;
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

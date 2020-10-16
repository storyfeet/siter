mod init;
//use gobble::StrungError;
use files::*;
use siter::err::*;
use siter::*;

//use toml::value::Table;
use clap_conf::*;
use config::*;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use templito::prelude::*;

fn main() -> anyhow::Result<()> {
    let clp = clap_app!(siter_gen =>
        (about:"A Program to generate a site from multiple forms of output")
        (version:crate_version!())
        (author:"Matthew Stoodley")
        (@arg root:-r --root +takes_value "The root of the project to work with")
        (@arg output:-o --output +takes_value "The folder to put the output default='public'")
        (@arg templates:-t --templates +takes_value ... "The list of folders to find templates in default='[templates]'")
        (@arg statics:-s --static +takes_value ... "The list of folders where to find static content default='[static]'")
        (@arg skip_static:--skip_static "skip static files")
        (@subcommand init => 
            (about:"Initialize directory as Siter project")
            (@arg folder:-f +takes_value "The folder to put the file in")
        )
    )
    .get_matches();

    let conf = &clp;
 
    match clp.subcommand(){
        ("init",Some(sub)) => return init::init(sub),
        _=>{},
    }

    //Get base Data
    let root = conf
        .grab_local()
        .arg("root")
        .def(std::env::current_dir()?.join("root_config.ito"));
    let root_folder = root.parent().ok_or(s_err("no parent folder for root"))?;

    let mut root_conf = RootConfig::new();
    root_conf.t_insert(TEMPLATES, &["templates"][..]);
    root_conf.t_insert(CONTENT, &["content"][..]);
    root_conf.t_insert("static", &["static"][..]);
    root_conf.t_insert("output", "public");
    root_conf.t_insert("root_folder", root_folder.display().to_string());
    root_conf.t_insert("root_file", root.display().to_string());
    root_conf.t_insert("ext", "html");
    root_conf.t_insert("type", "page.html");
    root_conf.t_insert("path_list", &["out_path"][..]);

    //Run the root config, this is where the main data is set

    let mut fman = default_func_man().with_exec().with_folder_lock(root_folder);
    let root_conf = load_root(&root, &root_conf, &mut NoTemplates, &fman)?;

    let op = root_conf.get("output").expect("No Ouput Path").to_string();
    fman = fman.with_write_lock(op);

    
    //setup for templito
    let mut tman = TMan::create(&root_conf)?;
    //let fman = default_func_man().with_exec();

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

        content_folder(&pb, &root_folder, &rc, &mut tman, &fman)?;
    }

    //build static
    if !clp.is_present("skip_static") {
        for c in root_conf
            .get_strs("static")
            .ok_or(s_err("Static folders not listed"))?
        {
            let pb = root_folder.join(c);
            println!("running static = {}", pb.display());
            static_folder(&pb, &root_folder, &root_conf, &mut tman, &fman)?;
        }
    }

    Ok(())
}

pub fn load_root<'a, P: AsRef<Path>, C: Configger, T: TempManager, F: FuncManager>(
    path: P,
    rc: &'a C,
    t: &mut T,
    f: &F,
) -> anyhow::Result<Config<'a>> {
    let root_temp = TreeTemplate::load(path)?;
    let (_, data) = root_temp.run_exp(&[rc], t, f)?;
    //println!("Root Data = {:?}", data);
    Ok(RootConfig::with_map(data).parent(rc))
}

pub fn content_folder(
    p: &Path,
    root: &Path,
    conf: &Config,
    tm: &mut TMan,
    fm: &BasicFuncs,
) -> anyhow::Result<()> {
    println!("processing content folder {}", p.display());
    let cpath = p.join("_config.ito");
    let conf = match load_root(cpath, conf, tm, fm) {
        Ok(v) => v,
        Err(_) => RootConfig::new().parent(conf),
    };
    for d in std::fs::read_dir(p)
        .wrap(format!("{}", p.display()))?
        .filter_map(|s| s.ok())
        .filter(|f| {
            !util::file_name(&f.path()).unwrap_or("_").starts_with("_")
                && match f.path().extension() {
                    Some(os) => os != ".ito",
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
            content_folder(&d.path(), root, &f_conf, tm, fm)?;
        } else if ft.is_file() {
            content_file(&d.path(), root, &f_conf, tm, fm)?;
        } else {
            //TODO Symlink
        }
    }

    Ok(())
}

pub fn content_file(
    p: &Path,
    root: &Path,
    conf: &dyn Configger,
    tm: &mut TMan,
    fm: &BasicFuncs,
) -> anyhow::Result<()> {
    let mut conf = Config::new(conf);
    let fstr = read_file(p)?;
    let ctt = TreeTemplate::from_str(&fstr)?;
    //run content file
    let (r_str, exp) = ctt.run_exp(&[&conf], tm, fm)?;

    for (k, v) in exp {
        conf.insert(k, v);
    }

    let base_t_name = conf.get_str("type").unwrap_or("page.html");

    let base_t = tm.get_t(base_t_name)?.clone();

    // run wrapper template
    let (out_str, base_exp) = base_t.run_exp(&[&conf, &r_str], tm, fm)?;
    for (k, v) in base_exp {
        conf.insert(k, v);
    }
    //Work out ouput file details and write

    let mut l_target = get_out_path(root, &conf)?;
    if conf.get("as_index").is_some() {
        let stem = l_target.file_stem().ok_or(Error::Str("No file to stem"))?;
        if stem != "index" {
            l_target = l_target.with_file_name(stem).join("index.html");
        }
    }
    let ext = conf.get_str("ext").unwrap_or("html");
    l_target = l_target.with_extension(ext);
    if let Some(par) = l_target.parent() {
        std::fs::create_dir_all(par)?;
    }
    println!("Outputting : {}", l_target.display());
    let mut f = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&l_target)?;
    write!(f, "{}", out_str).wrap(format!("Could not write {}", l_target.display()))?;
    Ok(())
}

pub fn static_folder<T: TempManager, F: FuncManager>(
    p: &Path,
    root: &Path,
    conf: &Config,
    tm: &mut T,
    fm: &F,
) -> anyhow::Result<()> {
    let cpath = p.join("_config.ito");
    let conf = match load_root(cpath, conf, tm, fm) {
        Ok(v) => v,
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
            static_folder(&d.path(), root, &f_conf, tm, fm)?;
        } else if ft.is_file() {
            let out_path = get_out_path(root, &f_conf)?;
            //Check target null or static newer
            if let (Ok(mto), Ok(mfr)) = (std::fs::metadata(&out_path), d.metadata()) {
                if let (Ok(tto), Ok(tfr)) = (mto.modified(), mfr.modified()) {
                    if tto > tfr {
                        continue;
                    }
                }
            }

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
    let out_file = conf.get_locked_str("output").unwrap_or("public");

    let mut l_target = root.join(out_file);
    l_target.push(target);
    Ok(l_target)
}

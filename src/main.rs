mod init;
mod m_exec;
//use gobble::StrungError;
use files::*;
use siter::*;
use err_tools::*;

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
        (@subcommand gen =>
            (@arg root:-r --root +takes_value "The root of the project to work with")
            (@arg output:-o --output +takes_value "The folder to put the output default='public'")
            (@arg templates:-t --templates +takes_value ... "The list of folders to find templates in default='[templates]'")
            (@arg statics:-s --static +takes_value ... "The list of folders where to find static content default='[static]'")
            (@arg skip_static:--skip_static "skip static files")
        )
        (@subcommand init => 
            (about:"Initialize directory as Siter project")
            (@arg folder:-f +takes_value "The folder to put the file in")
        )
        (@subcommand exec =>
            (about:"Run a template")
            (@arg t_file:-t + takes_value "File template")
            (@arg t_name:-n + takes_value "Named template")
            (@arg folder:-f + takes_value "The working folder")
            (@arg writefolder:-w + takes_value "The folder for output")
            (@arg data:-d + takes_value ... "Data equals sorted")
            (@arg rooted:-r "Works without folder locks")
        )
    )
    .get_matches();

 
    match &clp.subcommand(){
        ("init",Some(sub)) => init::init(sub),
        ("gen",Some(sub)) => gen(sub),
        ("exec",Some(sub)) => m_exec::exec(sub),
        _=>Ok(()),
    }

}

fn gen(conf : &ArgMatches) ->anyhow::Result<()>{
    
    //Get base Data
    let root = conf
        .grab_local()
        .arg("root")
        .def(std::env::current_dir()?.join("root_config.ito"));
    let root_folder = root.parent().e_str("no parent folder for root")?;

    let output = conf.grab_local().arg("output").def("public");

    let mut root_conf = RootConfig::new();
    root_conf.t_insert(TEMPLATES, &["templates"][..]);
    root_conf.t_insert(CONTENT, &["content"][..]);
    root_conf.t_insert("static", &["static"][..]);
    root_conf.t_insert("output", output.display().to_string());
    root_conf.t_insert("root_folder", root_folder.display().to_string());
    root_conf.t_insert("root_file", root.display().to_string());
    root_conf.t_insert("ext", "html");
    root_conf.t_insert("type", "page.html");
    root_conf.t_insert("path_list", &["out_path"][..]);

    //Run the root config, this is where the main data is set

    let mut fman = default_func_man().with_exec().with_folder_lock(root_folder);
    let mut bt = BasicTemps::new();
    let root_conf = load_root(&root, &root_conf, &mut bt, &fman)?;

    let op = root_conf.get("output").expect("No Ouput Path").to_string();
    println!("output = {}",op);
    fman = fman.with_write_lock(op);

    
    //setup for templito
    let mut tman = TMan::create(&root_conf)?;
    for (k,v) in bt {
        tman.insert_t(k,v)
    }
    //let fman = default_func_man().with_exec();

    //build content

    for c in root_conf
        .get_strs("content")
        .e_str("Content folders not listed")?
    {
        let rootbuf = root_folder.clone();
        let pb = rootbuf.join(&c);
        let mut rc = RootConfig::new().parent(&root_conf);
        rc.t_insert(CONTENT_FOLDER, c);
        rc.t_insert(CONTENT_FOLDER_PATH, pb.display().to_string());

        content_folder(&pb, &root_folder, &rc, &mut tman, &fman)?;
    }

    //build static
    if !conf.is_present("skip_static") {
        for c in root_conf
            .get_strs("static")
            .e_str("Static folders not listed")?
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
    let conf = match load_root(&cpath, conf, tm, fm) {
        Ok(v) => v,
        Err(e) =>{
            println!("Error in config file : {:?} : {}",cpath,e);
            RootConfig::new().parent(conf)
        }
    };
    for d in std::fs::read_dir(p)
        .e_string(format!("{}", p.display()))?
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
            util::file_name(&d.path()).e_str("File name no worky")?,
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
        let stem = l_target.file_stem().e_str("No file to stem")?;
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
    write!(f, "{}", out_str).e_string(format!("Could not write {}", l_target.display()))?;
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
        .e_string(p.display().to_string())?
        .filter_map(|s| s.ok())
    {
        println!("processing static folder entry {:?}", d);
        let ft = d.file_type()?;
        let mut f_conf = Config::new(&conf);
        f_conf.t_insert(
            "out_path",
            util::file_name(&d.path()).e_str("File name no worky")?,
        );
        if ft.is_dir() {
            static_folder(&d.path(), root, &f_conf, tm, fm)?;
        } else if ft.is_file() {
             
            let out_path = get_out_path(root, &f_conf)?;
            println!("Outpath = {}",out_path.display());
            //Check target null or static newer
            if let (Ok(mto), Ok(mfr)) = (std::fs::metadata(&out_path), d.metadata()) {
                if let (Ok(tto), Ok(tfr)) = (mto.modified(), mfr.modified()) {
                    if tto > tfr {
                        continue;
                    }
                }
            }

            if let Some(par) = out_path.parent() {
                println!("making dir {}",par.display());
                std::fs::create_dir_all(par)?;
            }
            println!("copying to {}",out_path.display());
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
        .e_str("No Path for Content")?;
    if target.is_absolute() {
        target = PathBuf::from(target.display().to_string().trim_start_matches("/"));
    }
    let out_file = conf.get_locked_str("output").unwrap_or("public");

    let mut l_target = root.join(out_file);
    l_target.push(target);
    Ok(l_target)
}

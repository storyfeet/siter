use crate::config::*;
use crate::*;
use err_tools::*;
use std::collections::HashMap;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use templito::prelude::*;

pub struct TMan {
    mp: HashMap<String, TreeTemplate>,
    paths: Vec<PathBuf>,
}

impl TMan {
    pub fn create(c: &Config) -> anyhow::Result<Self> {
        Ok(TMan {
            mp: HashMap::new(),
            paths: locale_paths(TEMPLATES, c)?,
        })
    }
}

impl TempManager for TMan {
    fn insert_t(&mut self, k: String, t: TreeTemplate) {
        self.mp.insert(k, t);
    }
    fn get_t(&mut self, k: &str) -> anyhow::Result<&TreeTemplate> {
        if self.mp.get(k) == None {
            let fstr = load_from_paths(&self.paths, k)?;
            let ftree = TreeTemplate::from_str(&fstr)?;
            self.mp.insert(k.to_string(), ftree);
        }
        self.mp.get(k).e_str("File not found")
    }
}

pub fn read_file<P: AsRef<Path>>(p: P) -> anyhow::Result<String> {
    let mut res = String::new();
    std::fs::File::open(p.as_ref())
        .e_string(p.as_ref().display().to_string())?
        .read_to_string(&mut res)?;
    Ok(res)
}

pub fn load_template(conf: &Config) -> anyhow::Result<String> {
    let name = conf.get_str("type").unwrap_or("page.html");
    load_locale(name, TEMPLATES, conf)
}

fn dollar(name: &str, conf: &Config) -> anyhow::Result<PathBuf> {
    if name.starts_with('$') {
        let pp = conf.get_built_path(OUT_PATH).e_str("No build path")?;
        let p2 = pp.parent().e_str("No Parent to build path")?;
        Ok(p2.join(name.trim_start_matches("$")))
    } else {
        Ok(PathBuf::from(name))
    }
}

pub fn locale_paths(lc: &str, conf: &Config) -> anyhow::Result<Vec<PathBuf>> {
    let root = conf.root_folder()?;
    Ok(conf
        .get_strs(lc)
        .e_str("No Template folders listed")?
        .into_iter()
        .map(|m| root.join(m))
        .collect())
}

pub fn load_from_paths(pp: &[PathBuf], name: &str) -> anyhow::Result<String> {
    for c in pp {
        let fp = c.join(name);
        match read_file(fp) {
            Ok(f) => return Ok(f),
            Err(_) => continue,
        };
    }
    e_str("File not Found")
}

pub fn load_locale(name: &str, locale: &str, conf: &Config) -> anyhow::Result<String> {
    let paths = locale_paths(locale, conf)?;
    let dname = dollar(name, conf)?;

    load_from_paths(&paths, &dname.display().to_string()).e_string(format!("{},{}", name, locale))
}

pub fn read_locale_dir(name: &str, locale: &str, conf: &Config) -> anyhow::Result<Vec<TData>> {
    let root = conf.root_folder()?;
    let np = dollar(name, conf)?;
    let mut res = Vec::new();
    let mut found = false;
    for c in conf.get_strs(locale).e_str("No Template folders listed")? {
        let fp = root.join(c).join(&np);
        match std::fs::read_dir(fp) {
            Ok(dir) => {
                found = true;
                res.extend(dir.filter_map(|s| s.ok()).filter_map(|m| {
                    m.path()
                        .file_name()
                        .and_then(|f| f.to_str().map(|s| TData::String(s.to_string())))
                }))
            }
            Err(_e) => {}
        }
    }
    match found {
        false => e_string(format!(
            "No {} folder found called '{}' from '{}'",
            locale,
            &np.display(),
            name
        )),
        true => Ok(res),
    }
}

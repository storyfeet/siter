use crate::config::*;
use crate::err::*;
use crate::*;
use std::io::Read;
use std::path::{Path, PathBuf};

pub fn read_file<P: AsRef<Path>>(p: P) -> anyhow::Result<String> {
    let mut res = String::new();
    std::fs::File::open(p.as_ref())
        .wrap(p.as_ref().display().to_string())?
        .read_to_string(&mut res)?;
    Ok(res)
}

pub fn load_template(conf: &Config) -> anyhow::Result<String> {
    let name = conf.get_str("type").unwrap_or("page.html");
    load_locale(name, TEMPLATES, conf)
}

fn dollar(name: &str, conf: &Config) -> anyhow::Result<PathBuf> {
    if name.starts_with('$') {
        let pp = conf
            .get_built_path(OUT_PATH)
            .ok_or(s_err("No build path"))?;
        let p2 = pp.parent().ok_or(s_err("No Parent to build path"))?;
        Ok(p2.join(name.trim_start_matches("$")))
    } else {
        Ok(PathBuf::from(name))
    }
}

pub fn load_locale(name: &str, locale: &str, conf: &Config) -> anyhow::Result<String> {
    let root = conf.root_folder()?;
    let np = dollar(name, conf)?;
    for c in conf
        .get_strs(locale)
        .ok_or(s_err("No Template folders listed"))?
    {
        let fp = root.join(c).join(&np);
        match read_file(fp) {
            Ok(f) => return Ok(f),
            Err(_) => continue,
        };
    }
    Err(Error::String(format!(
        "No {} for '{}' from '{}'",
        locale,
        np.display(),
        name
    ))
    .into())
}

pub fn read_locale_dir(
    name: &str,
    locale: &str,
    conf: &Config,
) -> anyhow::Result<Vec<toml::Value>> {
    let root = conf.root_folder()?;
    let np = dollar(name, conf)?;
    let mut res = Vec::new();
    let mut found = false;
    for c in conf
        .get_strs(locale)
        .ok_or(s_err("No Template folders listed"))?
    {
        let fp = root.join(c).join(&np);
        match std::fs::read_dir(fp) {
            Ok(dir) => {
                found = true;
                res.extend(dir.filter_map(|s| s.ok()).filter_map(|m| {
                    m.path()
                        .file_name()
                        .and_then(|f| f.to_str().map(|s| toml::Value::String(s.to_string())))
                }))
            }
            Err(_e) => {}
        }
    }
    match found {
        false => Err(err(format!(
            "No {} folder found called '{}' from '{}'",
            locale,
            &np.display(),
            name
        ))
        .into()),
        true => Ok(res),
    }
}

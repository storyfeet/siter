//use std::cell::RefCell;
use crate::{err::*, files::*, templates::pass, *};
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;
use std::fmt::Debug;
use std::fmt::Display;
use std::path::Path;
use std::path::PathBuf;

pub type TMap = HashMap<String, toml::Value>;
pub type GMap = HashMap<String, gtmpl::Value>;

pub trait Configger: Debug {
    fn get_built_path(&self, k: &str) -> Option<PathBuf>;
    fn get(&self, k: &str) -> Option<&toml::Value>;
    fn get_locked(&self, k: &str) -> Option<&toml::Value>;
    fn get_locked_str<'a>(&'a self, k: &str) -> Option<&'a str> {
        match self.get_locked(k)? {
            toml::Value::String(s) => Some(s),
            _ => None,
        }
    }
    fn get_str(&self, k: &str) -> Option<&str> {
        match self.get(k) {
            Some(toml::Value::String(s)) => Some(s),
            _ => None,
        }
    }
    fn get_strs(&self, k: &str) -> Option<Vec<&str>> {
        match self.get(k) {
            Some(toml::Value::Array(a)) => Some(
                a.iter()
                    .filter_map(|s| {
                        if let toml::Value::String(r) = s {
                            Some(&r[..])
                        } else {
                            None
                        }
                    })
                    .collect(),
            ),
            _ => None,
        }
    }
    fn root_folder(&self) -> anyhow::Result<PathBuf> {
        Ok(PathBuf::from(
            self.get_locked_str(ROOT_FOLDER)
                .ok_or(s_err("No Root Folder"))?,
        ))
    }

    fn to_gtmpl_map(&self) -> GMap;
    fn to_gtmpl(&self) -> gtmpl::Value {
        let mut res = self.to_gtmpl_map();
        if let Some(v) = self.get_strs("path_list") {
            for k in v {
                if let Some(bp) = self.get_built_path(k) {
                    res.insert(
                        format!("pp_{}", k),
                        gtmpl::Value::String(bp.display().to_string()),
                    );
                }
            }
        }
        gtmpl::Value::Map(res)
    }
}

#[derive(Debug)]
pub struct RootConfig {
    map: TMap,
}

impl RootConfig {
    pub fn new() -> Self {
        RootConfig {
            map: HashMap::new(),
        }
    }
    pub fn load<P: AsRef<Path>>(p: P) -> anyhow::Result<Self> {
        let ts = read_file(p)?;
        Ok(toml::from_str(&ts)?)
    }

    pub fn parent<'a>(self, parent: &'a dyn Configger) -> Config<'a> {
        Config {
            map: self.map,
            parent,
        }
    }
    pub fn from_map(map: HashMap<String, toml::Value>) -> Self {
        RootConfig { map }
    }
}

impl Configger for RootConfig {
    fn get_built_path(&self, k: &str) -> Option<PathBuf> {
        self.map.get(k).and_then(|tom| match tom {
            toml::Value::String(s) => Some(PathBuf::from(s)),
            _ => None,
        })
    }
    fn get(&self, k: &str) -> Option<&toml::Value> {
        self.map.get(k)
    }
    fn get_locked(&self, k: &str) -> Option<&toml::Value> {
        self.map.get(k)
    }
    fn to_gtmpl_map(&self) -> GMap {
        self.map
            .iter()
            .map(|(k, v)| (k.to_string(), pass::toml_to_gtmpl(v)))
            .collect()
    }
}

impl<'a> Configger for Config<'a> {
    fn get_built_path(&self, k: &str) -> Option<PathBuf> {
        let par_v = self.parent.get_built_path(k);
        let my_v = self.map.get(k).and_then(|tom| match tom {
            toml::Value::String(s) => Some(s),
            _ => None,
        });
        let res = match (par_v, my_v) {
            (Some(mut p), Some(v)) => {
                p.push(v);
                Some(p)
            }
            (Some(p), None) => Some(p),
            (None, Some(v)) => Some(PathBuf::from(v)),
            _ => None,
        };
        res
    }
    fn get(&self, k: &str) -> Option<&toml::Value> {
        match self.map.get(k) {
            Some(v) => Some(v),
            None => self.parent.get(k),
        }
    }
    fn get_locked(&self, k: &str) -> Option<&toml::Value> {
        self.parent.get_locked(k).or_else(|| self.map.get(k))
    }

    fn to_gtmpl_map(&self) -> GMap {
        let mut res = self.parent.to_gtmpl_map();
        for (k, v) in &self.map {
            res.insert(k.to_string(), pass::toml_to_gtmpl(v));
        }
        res
    }
}

#[derive(Debug)]
pub struct Config<'a> {
    parent: &'a dyn Configger,
    map: TMap,
}

impl<'a> Config<'a> {
    pub fn insert<K: Display, V>(&mut self, k: K, v: V)
    where
        toml::Value: From<V>,
    {
        self.map.insert(k.to_string(), toml::Value::from(v));
    }
}

impl<'a> Deserialize<'a> for RootConfig {
    fn deserialize<D: Deserializer<'a>>(deserializer: D) -> Result<Self, D::Error> {
        let map = HashMap::deserialize(deserializer)?;
        Ok(RootConfig { map })
    }
}

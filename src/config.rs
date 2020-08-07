//use std::cell::RefCell;
use crate::{err::*, files::*, *};
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;
use std::fmt::Debug;
use std::fmt::Display;
use std::path::Path;
use std::path::PathBuf;
use templito::prelude::*;
use templito::template::VarPart;
use templito::tparam::*;
use toml::Value as TV;

pub type TMap = HashMap<String, TV>;

pub trait Configger: Debug {
    fn get_built_path(&self, k: &str) -> Option<PathBuf>;
    fn get(&self, k: &str) -> Option<&TV>;
    fn get_locked(&self, k: &str) -> Option<&TV>;
    fn get_locked_str<'a>(&'a self, k: &str) -> Option<&'a str> {
        match self.get_locked(k)? {
            TV::String(s) => Some(s),
            _ => None,
        }
    }
    fn get_str(&self, k: &str) -> Option<&str> {
        match self.get(k) {
            Some(TV::String(s)) => Some(s),
            _ => None,
        }
    }
    fn get_strs(&self, k: &str) -> Option<Vec<&str>> {
        match self.get(k) {
            Some(TV::Array(a)) => Some(
                a.iter()
                    .filter_map(|s| {
                        if let TV::String(r) = s {
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

    fn insert(&mut self, k: String, v: TV);
}

pub trait TInserter {
    fn t_insert<K: Display, V>(&mut self, k: K, v: V)
    where
        TV: From<V>;
}

impl TInserter for dyn Configger {
    fn t_insert<K: Display, V>(&mut self, k: K, v: V)
    where
        TV: From<V>,
    {
        let k = k.to_string();
        let v = TV::from(v);
        self.insert(k, v);
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

    pub fn t_insert<K: Display, V>(&mut self, k: K, v: V)
    where
        TV: From<V>,
    {
        self.map.insert(k.to_string(), TV::from(v));
    }
    pub fn parent<'a>(self, parent: &'a dyn Configger) -> Config<'a> {
        Config {
            map: self.map,
            parent,
        }
    }
    pub fn from_map(map: HashMap<String, TV>) -> Self {
        RootConfig { map }
    }
}

impl Configger for RootConfig {
    fn get_built_path(&self, k: &str) -> Option<PathBuf> {
        self.map.get(k).and_then(|tom| match tom {
            TV::String(s) => Some(PathBuf::from(s)),
            _ => None,
        })
    }
    fn get(&self, k: &str) -> Option<&TV> {
        self.map.get(k)
    }
    fn get_locked(&self, k: &str) -> Option<&TV> {
        self.map.get(k)
    }
    fn insert(&mut self, k: String, v: TV) {
        self.map.insert(k, v);
    }
}

impl<'a> Configger for Config<'a> {
    fn get_built_path(&self, k: &str) -> Option<PathBuf> {
        let par_v = self.parent.get_built_path(k);
        let my_v = self.map.get(k).and_then(|tom| match tom {
            TV::String(s) => Some(s),
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
    fn get(&self, k: &str) -> Option<&TV> {
        match self.map.get(k) {
            Some(v) => Some(v),
            None => self.parent.get(k),
        }
    }
    fn get_locked(&self, k: &str) -> Option<&TV> {
        self.parent.get_locked(k).or_else(|| self.map.get(k))
    }

    fn insert(&mut self, k: String, v: TV) {
        self.map.insert(k, v);
    }
}

impl<'a> TParam for Config<'a> {
    fn get_v(&self, l: &[VarPart]) -> Option<TData> {
        if l.len() == 0 {
            return None;
        }
        let id = l[0].as_str()?;

        if l.len() == 1 {
            return self.get(id).map(|v| TData::from_toml(v.clone()));
        }
        match id {
            "lock" => self.get_locked(l[1].as_str()?)?.get_v(&l[2..]),
            "build" => Some(TData::String(
                self.get_built_path(l[1].as_str()?)?.display().to_string(),
            )),
            s => self.get(s)?.get_v(&l[1..]),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Config<'a> {
    parent: &'a dyn Configger,
    map: TMap,
}

impl<'a> Config<'a> {
    pub fn new(parent: &'a dyn Configger) -> Self {
        Config {
            parent,
            map: TMap::new(),
        }
    }
    pub fn t_insert<K: Display, V>(&mut self, k: K, v: V)
    where
        TV: From<V>,
    {
        self.map.insert(k.to_string(), TV::from(v));
    }
}

impl<'a> Deserialize<'a> for RootConfig {
    fn deserialize<D: Deserializer<'a>>(deserializer: D) -> Result<Self, D::Error> {
        let map = HashMap::deserialize(deserializer)?;
        Ok(RootConfig { map })
    }
}

pub fn tdata_to_toml(td: TData) -> TV {
    use TData::*;
    match td {
        Bool(b) => TV::Boolean(b),
        String(s) => TV::String(s),
        Int(i) => TV::Integer(i),
        UInt(u) => TV::Integer(u as i64),
        Float(f) => TV::Float(f),
        List(v) => TV::Array(v.into_iter().map(tdata_to_toml).collect()),
        Map(m) => TV::Table(m.into_iter().map(|(k, v)| (k, tdata_to_toml(v))).collect()),
        Null => TV::Boolean(false),
    }
}

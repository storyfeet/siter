use crate::{err::*, *};
use std::collections::HashMap;
use std::fmt::Debug;
use std::fmt::Display;
use std::path::PathBuf;
use templito::prelude::*;
use templito::template::VarPart;
use templito::tparam::*;

pub type TMap = HashMap<String, TData>;

pub trait Configger: Debug + TParam {
    fn get_built_path(&self, k: &str) -> Option<PathBuf>;
    fn get(&self, k: &str) -> Option<&TData>;
    fn get_locked(&self, k: &str) -> Option<&TData>;
    fn get_locked_str<'a>(&'a self, k: &str) -> Option<&'a str> {
        match self.get_locked(k)? {
            TData::String(s) => Some(s),
            _ => None,
        }
    }
    fn get_str(&self, k: &str) -> Option<&str> {
        match self.get(k) {
            Some(TData::String(s)) => Some(s),
            _ => None,
        }
    }
    fn get_strs(&self, k: &str) -> Option<Vec<&str>> {
        match self.get(k) {
            Some(TData::List(a)) => Some(
                a.iter()
                    .filter_map(|s| {
                        if let TData::String(r) = s {
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

    fn insert(&mut self, k: String, v: TData);
}

pub trait TInserter {
    fn t_insert<K: Display, V>(&mut self, k: K, v: V)
    where
        TData: From<V>;
}

impl TInserter for dyn Configger {
    fn t_insert<K: Display, V>(&mut self, k: K, v: V)
    where
        TData: From<V>,
    {
        let k = k.to_string();
        let v = TData::from(v);
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

    pub fn with_map(map: TMap) -> Self {
        RootConfig { map }
    }

    pub fn t_insert<K: Display, V>(&mut self, k: K, v: V)
    where
        TData: From<V>,
    {
        self.map.insert(k.to_string(), TData::from(v));
    }
    pub fn parent<'a>(self, parent: &'a dyn Configger) -> Config<'a> {
        Config {
            map: self.map,
            parent,
        }
    }
    pub fn from_map(map: HashMap<String, TData>) -> Self {
        RootConfig { map }
    }
}

impl Configger for RootConfig {
    fn get_built_path(&self, k: &str) -> Option<PathBuf> {
        self.map.get(k).and_then(|tom| match tom {
            TData::String(s) => Some(PathBuf::from(s)),
            _ => None,
        })
    }
    fn get(&self, k: &str) -> Option<&TData> {
        self.map.get(k)
    }
    fn get_locked(&self, k: &str) -> Option<&TData> {
        self.map.get(k)
    }
    fn insert(&mut self, k: String, v: TData) {
        self.map.insert(k, v);
    }
}

impl<'a> Configger for Config<'a> {
    fn get_built_path(&self, k: &str) -> Option<PathBuf> {
        let par_v = self.parent.get_built_path(k);
        let my_v = self.map.get(k).and_then(|tom| match tom {
            TData::String(s) => Some(s),
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
    fn get(&self, k: &str) -> Option<&TData> {
        match self.map.get(k) {
            Some(v) => Some(v),
            None => self.parent.get(k),
        }
    }
    fn get_locked(&self, k: &str) -> Option<&TData> {
        self.parent.get_locked(k).or_else(|| self.map.get(k))
    }

    fn insert(&mut self, k: String, v: TData) {
        self.map.insert(k, v);
    }
}

impl TParam for RootConfig {
    fn get_v<'a>(&'a self, l: &[VarPart]) -> Option<TBoco<'a>> {
        if l.len() == 0 {
            return None;
        }
        let id = l[0].as_str()?;

        if l.len() == 1 {
            return self.get(id).map(|v| TBoco::Bo(v));
        }
        match id {
            "lock" => self.get_locked(l[1].as_str()?)?.get_v(&l[2..]),
            "build" => Some(TBoco::Co(TData::String(
                self.get_built_path(l[1].as_str()?)?.display().to_string(),
            ))),
            s => self.get(s)?.get_v(&l[1..]),
        }
    }
}

impl<'a> TParam for Config<'a> {
    fn get_v<'b>(&'b self, l: &[VarPart]) -> Option<TBoco<'b>> {
        if l.len() == 0 {
            return None;
        }
        let id = l[0].as_str()?;

        if l.len() == 1 {
            return self.get(id).map(|v| TBoco::Bo(v));
        }
        match id {
            "lock" => self.get_locked(l[1].as_str()?)?.get_v(&l[2..]),
            "build" => Some(TBoco::Co(TData::String(
                self.get_built_path(l[1].as_str()?)?.display().to_string(),
            ))),
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
        TData: From<V>,
    {
        self.map.insert(k.to_string(), TData::from(v));
    }
}

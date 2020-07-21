//use std::cell::RefCell;
use crate::{templates::pass, util};
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;
use std::fmt::Display;
use std::path::Path;
use std::path::PathBuf;
use std::rc::Rc;

pub type CMap = HashMap<String, toml::Value>;

pub struct Config {
    parent: Option<Rc<Config>>,
    map: CMap,
}

impl Config {
    pub fn new() -> Self {
        Config {
            parent: None,
            map: HashMap::new(),
        }
    }

    pub fn rc_with<K: Display, V>(self: Rc<Self>, k: K, v: V) -> Rc<Self>
    where
        toml::Value: From<V>,
    {
        let mut res = Self::new().parent(self.clone());
        res.insert(k, v);
        Rc::new(res)
    }

    pub fn from_map(map: HashMap<String, toml::Value>) -> Self {
        Config { parent: None, map }
    }

    pub fn load<P: AsRef<Path>>(p: P) -> anyhow::Result<Self> {
        let ts = util::read_file(p)?;
        Ok(toml::from_str(&ts)?)
    }

    pub fn parent(mut self, p: Rc<Config>) -> Self {
        self.parent = Some(p);
        self
    }

    pub fn insert<K: Display, V>(&mut self, k: K, v: V)
    where
        toml::Value: From<V>,
    {
        self.map.insert(k.to_string(), toml::Value::from(v));
    }

    pub fn get<K: AsRef<str>>(&self, k: K) -> Option<&toml::Value> {
        match self.map.get(k.as_ref()) {
            Some(v) => Some(v),
            _ => match &self.parent {
                Some(r) => r.get(k),
                None => None,
            },
        }
    }

    pub fn get_str<K: AsRef<str>>(&self, k: K) -> Option<&str> {
        match self.map.get(k.as_ref()) {
            Some(toml::Value::String(s)) => Some(s),
            _ => match &self.parent {
                Some(r) => r.get_str(k),
                None => None,
            },
        }
    }
    pub fn get_strs<K: AsRef<str>>(&self, k: K) -> Option<Vec<&str>> {
        match self.map.get(k.as_ref()) {
            Some(toml::Value::Array(ar)) => {
                let mut res = Vec::new();
                for a in ar {
                    if let toml::Value::String(ref s) = a {
                        res.push(&s[..]);
                    }
                }
                Some(res)
            }
            _ => match &self.parent {
                Some(r) => r.get_strs(k),
                None => None,
            },
        }
    }

    pub fn get_built_path<K: AsRef<str>>(&self, k: K) -> Option<PathBuf> {
        let bp = match &self.parent {
            Some(p) => p.get_built_path(k.as_ref()),
            None => None,
        };
        match (bp, self.get_str(k.as_ref())) {
            (Some(mut p), Some(v)) => {
                p.push(v);
                Some(p)
            }
            (Some(p), None) => Some(p),
            (None, Some(v)) => Some(PathBuf::from(v)),
            _ => None,
        }
    }

    pub fn gtmpl_map(&self) -> HashMap<String, gtmpl::Value> {
        //TODO work out how to handle paths
        let mut res = match &self.parent {
            Some(p) => p.gtmpl_map(),
            None => HashMap::new(),
        };
        for (k, v) in &self.map {
            res.insert(k.to_string(), pass::toml_to_gtmpl(v));
        }
        res
    }
}

impl<'a> Deserialize<'a> for Config {
    fn deserialize<D: Deserializer<'a>>(deserializer: D) -> Result<Self, D::Error> {
        let map = HashMap::deserialize(deserializer)?;
        Ok(Config { parent: None, map })
    }
}

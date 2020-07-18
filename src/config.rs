//use std::cell::RefCell;
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;
use std::fmt::Display;
use std::io::Read;
use std::path::Path;
use std::rc::Rc;
pub struct Config {
    parent: Option<Rc<Config>>,
    map: HashMap<String, toml::Value>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            parent: None,
            map: HashMap::new(),
        }
    }

    pub fn load<P: AsRef<Path>>(p: P) -> anyhow::Result<Self> {
        let mut ts = String::new();
        std::fs::File::open(p)?.read_to_string(&mut ts)?;
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
}

impl<'a> Deserialize<'a> for Config {
    fn deserialize<D: Deserializer<'a>>(deserializer: D) -> Result<Self, D::Error> {
        let map = HashMap::deserialize(deserializer)?;
        Ok(Config { parent: None, map })
    }
}

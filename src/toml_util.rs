use std::io::Read;
use std::path::Path;

pub fn to_table(v: toml::Value) -> anyhow::Result<toml::value::Table> {
    match v {
        toml::Value::Table(t) => Ok(t),
        _ => Err(crate::err::Error::TomlNotMap.into()),
    }
}

pub fn as_table(v: &toml::Value) -> anyhow::Result<&toml::value::Table> {
    match v {
        toml::Value::Table(t) => Ok(t),
        _ => Err(crate::err::Error::TomlNotMap.into()),
    }
}

pub fn as_table_mut(v: &mut toml::Value) -> anyhow::Result<&mut toml::value::Table> {
    match v {
        toml::Value::Table(t) => Ok(t),
        _ => Err(crate::err::Error::TomlNotMap.into()),
    }
}

pub fn as_arr(v: &toml::Value) -> anyhow::Result<&Vec<toml::Value>> {
    match v {
        toml::Value::Array(t) => Ok(t),
        _ => Err(crate::err::Error::TomlNotMap.into()),
    }
}

pub fn load_toml<P: AsRef<Path>>(p: P) -> anyhow::Result<toml::Value> {
    let mut ts = String::new();
    std::fs::File::open(p)?.read_to_string(&mut ts)?;
    Ok(ts.parse()?)
}

pub fn child_config<P: AsRef<Path>>(parent: &toml::Value, p: P) -> anyhow::Result<toml::Value> {
    let mut t2 = load_toml(p)?;
    let tab2 = as_table_mut(&mut t2)?;

    for (k, v) in as_table(&parent)?.iter() {
        if !tab2.get(k).is_some() {
            tab2.insert(k.clone(), v.clone());
        }
    }
    Ok(t2)
}

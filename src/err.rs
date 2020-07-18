use thiserror::*;
#[derive(Error, Clone, Debug, PartialEq)]
pub enum Error {
    #[error("Toml Result from Frontmatter must be a header")]
    TomlNotMap,
    #[error("No Command supplied for exec")]
    NoExecCommand,
    #[error("{}",.0)]
    Str(&'static str),
    #[error("{}",.0)]
    String(String),
}

pub fn s_err(s: &'static str) -> Error {
    Error::Str(s)
}

#[derive(Error, Debug)]
#[error("{} -- at -- {}",.e,.s)]
pub struct EWrap {
    s: String,
    e: anyhow::Error,
}

impl EWrap {
    pub fn new(s: String, e: anyhow::Error) -> Self {
        EWrap { s, e }
    }
}

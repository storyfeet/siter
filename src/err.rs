use thiserror::*;
#[derive(Error, Clone, Debug, PartialEq)]
pub enum Error {
    #[error("File Not Found")]
    FileNotFound,
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

pub fn err(s: String) -> Error {
    Error::String(s)
}

pub fn s_wrap<T>(r: anyhow::Result<T>, s: String) -> anyhow::Result<T> {
    r.map_err(|e| EWrap { s, e }.into())
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

pub trait ERes<A, T>: Sized {
    fn wrap(self, s: String) -> Result<A, EWrap>;
    fn s_wrap(self, s: &str) -> Result<A, EWrap> {
        self.wrap(s.to_string())
    }
}

impl<A, T: Into<anyhow::Error>> ERes<A, T> for Result<A, T> {
    fn wrap(self, s: String) -> Result<A, EWrap> {
        self.map_err(|e| EWrap {
            s: s.to_string(),
            e: e.into(),
        })
    }
}

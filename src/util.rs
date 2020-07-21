use crate::err::ERes;
use std::io::Read;
use std::path::Path;

pub fn read_file<P: AsRef<Path>>(p: P) -> anyhow::Result<String> {
    let mut res = String::new();
    std::fs::File::open(p.as_ref())
        .wrap(p.as_ref().display().to_string())?
        .read_to_string(&mut res)?;
    Ok(res)
}

pub fn file_name(p: &Path) -> Option<&str> {
    match p.file_name() {
        Some(s) => s.to_str(),
        _ => None,
    }
}

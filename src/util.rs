use std::io::Read;
use std::path::Path;

pub fn read_file<P: AsRef<Path>>(p: P) -> anyhow::Result<String> {
    let mut res = String::new();
    std::fs::File::open(p)?.read_to_string(&mut res)?;
    Ok(res)
}

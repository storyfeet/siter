use clap::ArgMatches;
use std::io::Write;
use std::path::Path;

const BINDEX: &str = r#"{{export title="Index"}}{{@md}}
Index Page
========

{{/md}}"#;

const BTEMPLATE: &str = r#"<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<title>{{first .title "Default Title"}}<title>
</head>
<body>
{{$1}}
</body>
</html>
"#;

fn try_create<F: AsRef<Path>>(fname: F, data: &str) -> anyhow::Result<()> {
    if fname.as_ref().exists() {
        return Ok(());
    }
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .open(fname)?;
    write!(f, "{}", data).map_err(|e| e.into())
}

pub fn init(conf: &ArgMatches) -> anyhow::Result<()> {
    let f = std::path::PathBuf::from(conf.value_of("folder").unwrap_or(""));

    std::fs::create_dir(&f)?;
    std::fs::create_dir(f.join("content"))?;
    std::fs::create_dir(f.join("templates"))?;
    std::fs::create_dir(f.join("static"))?;
    try_create(f.join("root_config.ito"), "")?;
    try_create(f.join("content/index.md"), BINDEX)?;
    try_create(f.join("templates/page.html"), BTEMPLATE)?;
    Ok(())
}

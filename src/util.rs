use std::path::Path;

pub fn file_name(p: &Path) -> Option<&str> {
    match p.file_name() {
        Some(s) => s.to_str(),
        _ => None,
    }
}

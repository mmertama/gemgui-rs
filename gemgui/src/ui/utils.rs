use std::path::Path;


pub (crate) fn html_file_launch_cmd() -> Option<String> {
    if cfg!(target_os = "unix") {
        return Some("x-www-browser".to_string());
    }
    if cfg!(target_os = "macos") {
        return Some("open".to_string());
    }
    if cfg!(target_os = "windows") {
        return Some("start /max".to_string());
    }
    eprintln!("Unknown OS");
    None
}

pub (crate) fn get_extension_from_filename(filename: &str) -> Option<&str> {
    Path::new(filename)
        .extension()
        .and_then(std::ffi::OsStr::to_str)
}

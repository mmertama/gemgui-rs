use std::path::Path;


pub (crate) fn html_file_launch_cmd() -> Option<(String, Vec<String>)> {
    if cfg!(target_os = "unix") || cfg!(target_os = "linux") {
        return Some(("x-www-browser".to_string(), Vec::new()));
    }
    if cfg!(target_os = "macos") {
        return Some(("open".to_string(), Vec::new()));
    }
    if cfg!(target_os = "windows") {
        return Some(("cmd".to_string(), vec!("/C", "start /max").iter().map(|s|{s.to_string()}).collect()));
    }
    eprintln!("Unknown OS");
    None
}

pub (crate) fn get_extension_from_filename(filename: &str) -> Option<&str> {
    Path::new(filename)
        .extension()
        .and_then(std::ffi::OsStr::to_str)
}

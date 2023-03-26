use std::path::Path;
use which::which;
use std::path::PathBuf;
use std::process::{Command, Stdio};


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


pub (crate) fn python3() -> Option<PathBuf>  {
    let py3 = which("python3");
    if let Ok(py) = py3 {
        return Some(py);
    }
    let py = which("python");
    if py.is_err() {
        return None;
    }
    let output = Command::new("python").arg("--version").stdout(Stdio::piped()).output();
    if output.is_err() {
        return None;
    }
    let out = String::from_utf8(output.unwrap().stdout);
    if out.is_err() {
        return None;
    }
    let out = out.unwrap();
    let pv = out.split(' ').collect::<Vec<&str>>(); //// Python 2.7.16
    if pv.len() < 2 {
        return None;
    }
    let ver = pv[1].split('.').collect::<Vec<&str>>();
    if pv.is_empty() {
        return None;
    }
    let major = ver[0].parse::<u32>();
    if major.is_err() || major.unwrap() < 3 {
        return None;
    }
    Some(py.unwrap())
}

#[allow(dead_code)]
pub (crate) fn type_of<T>(_: &T) -> String {
    std::any::type_name::<T>().to_string()
}

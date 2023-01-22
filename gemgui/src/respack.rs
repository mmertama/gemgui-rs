use std::path::Path;
use cargo_metadata;


#[allow(dead_code)]
fn html_metadata(toml_path: &str)->Option<String> {
    let path = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let meta = match cargo_metadata::MetadataCommand::new()
    .manifest_path(toml_path)
    .current_dir(&path)
    .exec()  {
        Ok(o) => Some(o),
        Err(e) => {
            eprintln!("Metadata error {}", e);
            None
        }
    }?;
    
    let root = meta.root_package().unwrap();
    Some(String::from(root.metadata["html"].as_str()?))
}

fn get_extension_from_filename(filename: &str) -> Option<&str> {
    std::path::Path::new(filename)
        .extension()
        .and_then(std::ffi::OsStr::to_str)
}

fn minify(name : &str, data : &[u8]) -> Vec<u8> {
    let minify_js = || {
        let source = String::from_utf8_lossy(data);
        let minified = minifier::js::minify(&source);
        minified.to_string().as_bytes().to_vec()
    };
    let minify_json = || {
        let source = String::from_utf8_lossy(data);
        let minified = minifier::json::minify(&source);
        minified.to_string().as_bytes().to_vec()
    };
    let minify_css = || {   
        let source = String::from_utf8_lossy(data);
        let minified = minifier::css::minify(&source).unwrap();
        minified.to_string().as_bytes().to_vec()
    };
    let ext =  get_extension_from_filename(name);
    if ext.is_none() {
        return data.to_owned();
    }

    match ext.unwrap() {
        "js" => minify_js(), 
        "json" => minify_json(),
        "css" => minify_css(),
        _ => data.to_owned()
    }
}

/// Pack a folder as a resources
/// Assumed to be called at build.rs, i.e. build time only
/// Generates a source file "generated.rs" that can be 
/// included in the application sources and be loaded
/// as a filemap for a dynamic access.
///
/// Add in Cargo.toml
/// ```javascript
/// [build-dependencies]
/// gemgui = "...""
/// ```
/// ...and apply build.rs 
/// 'gui' refers to directory the resources all locate. All files in the directory
/// are read in resources.
/// ```no_run
///  # #[cfg(never)] mod foo {
/// fn main() {
///     println!("cargo:rerun-if-changed=gui");
///     gemgui::respack::pack("gui", false);
/// }
/// # }
/// ```
/// Then resources can be read in sources as
/// ```no_run
/// # #[cfg(never)] mod foo {
/// include!(concat!(env!("OUT_DIR"), "/generated.rs"));
/// # }
/// ```
/// ...and read in for the Ui as  
/// ```no_run
/// # #[cfg(never)] mod foo {
/// # fn bar() {
/// let fm = gemgui::filemap_from(RESOURCES);
/// let mut ui = Ui::new(fm, "hello.html", 12345)?;
/// # }
/// # }
/// ```
/// 
///  # Arguments
///  
/// ´directory´ - Directory to read in resources. Does not read sub directories.
/// 
/// ´try_minify´ -  May to apply minify / compress.
/// 
pub fn pack<PathStr>(directory: PathStr, try_minify: bool) 
where PathStr: AsRef<Path> {
    let out_dir = std::env::var_os("OUT_DIR").unwrap();
    let dest_path = std::path::Path::new(&out_dir).join("generated.rs");
    let mut lines = Vec::new();
    let mut names = Vec::new();
    let dir = std::fs::read_dir(directory).unwrap();
    for entry in dir {
        let file = entry.unwrap();
        if file.file_type().unwrap().is_file() {
            let content_raw = std::fs::read(file.path()).unwrap();
            let name = file.file_name().into_string().unwrap(); 
            if name.starts_with('.') {
                continue; // no hidden files (mainly due .DS_Store in mac)
            }
            let content = if try_minify {
                minify(&name, &content_raw)
            } else  {
                content_raw
            };
            let coded = base64::encode(content);
            let re = regex::Regex::new(r"[^A-Za-z_0-9]").unwrap();
            let result = re.replace_all(&name, "_");
            let re = regex::Regex::new(r"^[^A-Za-z_]").unwrap();
            let result = re.replace_all(&result, "_");
            let rust_name = result.to_ascii_uppercase();

            let line = format!("const {}: &str = {:#?};", &rust_name, coded);
            lines.push(line);
            names.push((name.to_owned(), rust_name.to_string()));
        }
    }

    lines.push("const RESOURCES: &[(&str, &str)] = &[".to_string());
    for pair in names {
        lines.push(format!("({:#?}, {}),", pair.0, pair.1));
    }
    lines.push("];".to_string());
    println!("{} --> {:?}", lines.len(), lines);
    std::fs::write(&dest_path, lines.join("\n")).unwrap();
}

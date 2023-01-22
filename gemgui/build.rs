
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/respack.rs"));

fn main() {
    println!("cargo:rerun-if-changed=build.rs"); 
    const GEMGUI_PATH: &str = "js/gemgui.js";
    println!("cargo:rerun-if-changed={}", GEMGUI_PATH);
    pack("js", false);
    }
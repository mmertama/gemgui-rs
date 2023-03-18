
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/respack.rs"));

fn main() {
    println!("cargo:rerun-if-changed=build.rs"); 
    const GEMGUI_PATH: &str = "res/gemgui.js";
    const PYGUI_PATH: &str = "res/pyclient.py";
    println!("cargo:rerun-if-changed={GEMGUI_PATH}");
    println!("cargo:rerun-if-changed={PYGUI_PATH}");
    pack("res", false);
    }
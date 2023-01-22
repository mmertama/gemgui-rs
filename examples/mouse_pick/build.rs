

fn main() {
    println!("cargo:rerun-if-changed=gui");
    gemgui::respack::pack("gui", false);
    }
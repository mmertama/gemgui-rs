#[path="./chrome.rs"]
mod chrome;
use std::path::PathBuf;
use std::sync::Once;
use std::panic;
use std::time::Duration;


#[allow(unused)]
fn initialize() { 
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        panic::set_hook(Box::new(|e| {
            let killed = chrome::kill_headless();
            println!("Test panic {}, headless killed: {}", e, killed);
            std::process::exit(1);
        }));
    });
    
}

#[allow(unused)]
pub (crate) fn setup () -> gemgui::ui::Gui { 
        initialize();
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("tests/assets");
        let fm = gemgui::filemap_from_dir(&path).unwrap();
        let port = 30000u16;
        chrome::kill_headless();
        while !gemgui::wait_free_port(port, Duration::from_secs(2)) {
            chrome::kill_headless();
        }
        let mut ui = gemgui::ui::Gui::new(fm, "tests.html", port).unwrap();
        let chrome = chrome::system_chrome();
        if chrome.is_some() {
            let (cmd, cmd_params) = chrome.unwrap();
            let mut params = cmd_params.clone();
            let mut chrome_params = chrome::headless_params(false);
            params.append(&mut chrome_params);
            params.push(ui.address());
            ui.set_gui_command_line(&cmd, &params);
            }
        ui
        }



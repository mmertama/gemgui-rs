#[path="./chrome.rs"]
mod chrome;
use std::sync::Once;
use std::panic;
use std::time::Duration;


#[allow(unused)]
fn initialize() {
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        panic::set_hook(Box::new(|e| {
            let killed = chrome::kill_headless();
            println!("Test panic {}, headlerss killed: {}", e, killed);
            std::process::exit(1);
        }));
    });
}

#[allow(unused)]
pub (crate) fn setup () -> gemgui::ui::Gui { 
        initialize();
        let path = std::path::Path::new("tests/assets");
        let fm = gemgui::filemap_from_dir(&path).unwrap();
        let port = 30000u16;
        chrome::kill_headless();
        while !gemgui::wait_free_port(port, Duration::from_secs(2)) {
            chrome::kill_headless();
        }
        let mut ui = gemgui::ui::Gui::new(fm, "tests.html", port).unwrap();
        let chrome = chrome::system_chrome();
        if chrome.is_some() {
            ui.set_gui_command_line(&chrome.unwrap(), &chrome::headless_params(false));
            }
        ui
        }

/*
#[macro_export]
macro_rules! setup {
    () => { {
        initialize();
        let path = std::path::Path::new("tests/assets");
        let fm = gemgui::filemap_from_dir(&path).unwrap();
        let port = 30000u16;
        chrome::kill_headless();
        while(!gemgui::wait_free_port(port, Duration::from_secs(2))) {
            chrome::kill_headless();
        }
        let mut ui = gemgui::ui::Ui::new(fm, String::from("tests.html"), port).unwrap();
        let chrome = chrome::system_chrome();
        if chrome.is_some() {
            ui.set_gui_command_line(chrome.unwrap(), chrome::headless_params(false));
            }
        ui
        }
    };
*/



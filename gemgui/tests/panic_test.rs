use std::path::Path;
use serial_test::serial;

use gemgui::ui::Gui;

#[tokio::test]
#[should_panic]
#[serial]
async fn test_folder_not_found() {
    let path = Path::new("tests/not_found");
    gemgui::filemap_from_dir(&path).unwrap();
}

#[tokio::test]
#[should_panic]
#[serial]
async fn test_entry_page_not_found() {
    let path = Path::new("tests/assets");
    let fm = gemgui::filemap_from_dir(&path).unwrap();
    let port = gemgui::next_free_port(30000u16);
    Gui::new(fm, "not_found.html", port).unwrap();
}

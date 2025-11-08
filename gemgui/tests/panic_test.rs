use std::path::Path;
use serial_test::serial;

use gemgui::ui::Gui;

#[tokio::test]
#[serial]
async fn test_folder_not_found() {
    let path = Path::new("tests/not_found");
    let err = gemgui::filemap_from_dir(&path).expect_err("");
    assert_eq!(err.kind(), std::io::ErrorKind::NotFound);
}

#[tokio::test]
#[should_panic]
#[serial]
async fn test_entry_page_not_found() {
    let path = Path::new("tests/assets");
    let fm = gemgui::filemap_from_dir(&path).unwrap();
    let port = gemgui::next_free_port(30000u16);
    let result = Gui::new(fm, "not_found.html", port);
    assert!(result.is_err(), "It should not found");
}

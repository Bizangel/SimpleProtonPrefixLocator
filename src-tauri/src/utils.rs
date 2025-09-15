use std::io;
use std::path;
use std::process::Command;

#[cfg(target_os = "linux")]
pub fn xdg_open_folder(folder_path: &path::Path) -> io::Result<std::process::Child> {
    Command::new("xdg-open").arg(folder_path).spawn()
}

pub fn mime_from_extension(path: &str) -> &'static str {
    if path.ends_with(".png") {
        "image/png"
    } else if path.ends_with(".jpg") || path.ends_with(".jpeg") {
        "image/jpeg"
    } else if path.ends_with(".ico") {
        "image/x-icon"
    } else if path.ends_with(".gif") {
        "image/gif"
    } else {
        "application/octet-stream"
    }
}

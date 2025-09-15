use std::env;
use std::fs;
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

pub fn get_steam_path() -> Result<path::PathBuf, String> {
    let user_home_dir = env::home_dir().ok_or("Could not find home directory!".to_string())?;
    let steam_path = user_home_dir.join(".local/share/Steam");
    if !steam_path.is_dir() {
        return Err("Steam folder not found! Is steam installed?".to_string());
    }

    Ok(steam_path)
}

pub fn get_all_steam_user_ids(steam_path: &path::PathBuf) -> Result<Vec<String>, String> {
    let user_ids: Vec<String> = fs::read_dir(steam_path.join("userdata"))
        .map_err(|x| x.to_string())?
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| entry.file_name().into_string().ok())
        .collect();
    Ok(user_ids)
}

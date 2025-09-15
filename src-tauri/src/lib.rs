// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod binary_vdf_parser;
mod proton_app;

use proton_app::ProtonApp;

use base64::{engine::general_purpose, Engine as _};
use binary_vdf_parser::VdfMap;
use std::env;
use std::fs;
use std::io;
use std::path;
use std::process::Command;
use tauri::Manager;

fn mime_from_extension(path: &str) -> &'static str {
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

struct AppState {
    parsed_shortcuts: Result<Vec<ProtonApp>, String>,
}

#[tauri::command]
fn read_steam_vdf_shortcuts(state: tauri::State<AppState>) -> String {
    return match &state.parsed_shortcuts {
        Ok(parsed_shortcuts) => serde_json::to_string_pretty(parsed_shortcuts).unwrap(),
        Err(e) => return format!("{{ \"error\": \"{}\"}}", e),
    };
}

#[cfg(target_os = "linux")]
fn xdg_open_folder(folder_path: &path::Path) -> io::Result<std::process::Child> {
    Command::new("xdg-open").arg(folder_path).spawn()
}

#[tauri::command]
fn open_appid_prefix(state: tauri::State<AppState>, appid: &str) -> Result<(), String> {
    let parsed = state.parsed_shortcuts.as_ref()?;
    if !parsed.iter().any(|x| x.appid == appid) {
        return Err("Invalid AppID".to_string());
    }

    let user_home_dir = env::home_dir().ok_or_else(|| "Unable to fetch $HOME".to_string())?;
    let prefix_path = user_home_dir
        .join(".local/share/Steam")
        .join(format!("steamapps/compatdata/{}/pfx/drive_c", appid));

    let _ = xdg_open_folder(&prefix_path); // doesn't matter if error
    Ok(())
}

fn get_protonapps_from_shortcuts_vdf(vdf: &VdfMap) -> Result<Vec<ProtonApp>, String> {
    let shortcuts = vdf
        .get("shortcuts")
        .ok_or_else(|| "Unable to parse shortcuts".to_string())?
        .as_map()
        .ok_or_else(|| "Unable to parse shortcuts".to_string())?;

    let apps = shortcuts
        .values()
        .filter_map(|entry| {
            let entry = entry.as_map()?;
            let icon_path = entry.get("icon")?.as_str()?;
            // Build icon string only if path is non-empty
            let icon = if icon_path.is_empty() {
                String::new()
            } else {
                match fs::read(icon_path) {
                    Ok(bytes) => {
                        let icon_b64 = general_purpose::STANDARD.encode(bytes);
                        let icon_mime = mime_from_extension(icon_path);
                        format!("data:{};base64,{}", icon_mime, icon_b64)
                    }
                    Err(_) => String::new(), // include app, but no icon
                }
            };

            Some(ProtonApp {
                appid: entry.get("appid")?.copy_as_str()?,
                appname: entry.get("appname")?.copy_as_str()?,
                icon: icon,
                lastplaytime: entry.get("lastplaytime")?.copy_as_str()?,
                exe: entry.get("exe")?.copy_as_str()?,
                startdir: entry.get("startdir")?.copy_as_str()?,
            })
        })
        .collect();

    Ok(apps)
}

fn get_protonapps_from_vdf_shortcuts() -> Result<Vec<ProtonApp>, String> {
    let user_home_dir = env::home_dir().ok_or("Could not find home directory!".to_string())?;
    let steam_path = user_home_dir.join(".local/share/Steam");

    if !steam_path.is_dir() {
        return Err("Steam folder not found! Is steam installed?".to_string());
    }

    let user_ids: Vec<String> = fs::read_dir(steam_path.join("userdata"))
        .map_err(|x| x.to_string())?
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| entry.file_name().into_string().ok())
        .collect();

    let vdfmaps: Vec<VdfMap> = user_ids
        .iter()
        .map(|userid| steam_path.join(format!("userdata/{}/config/shortcuts.vdf", &userid)))
        .filter(|path| path.is_file())
        .filter_map(|path| {
            fs::read(&path)
                .ok()
                .and_then(|data| binary_vdf_parser::read_vdf(data).ok())
        })
        .collect();

    let parsed: Vec<ProtonApp> = vdfmaps
        .iter()
        .filter_map(|x| get_protonapps_from_shortcuts_vdf(&x).ok())
        .flatten()
        .collect();

    Ok(parsed)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let parsed_vdfs = get_protonapps_from_vdf_shortcuts();

    tauri::Builder::default()
        .setup(|app| {
            app.manage(AppState {
                parsed_shortcuts: parsed_vdfs,
            });
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            open_appid_prefix,
            read_steam_vdf_shortcuts
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

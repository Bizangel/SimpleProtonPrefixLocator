// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod vdf_parser;

use base64::{engine::general_purpose, Engine as _};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io;
use std::process::Child;
use std::process::Command;
use tauri::Manager;
use vdf_parser::VdfMap;

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
    parsed_shortcuts: Result<HashMap<String, VdfMap>, String>,
}

#[tauri::command]
fn read_steam_vdf_shortcuts(state: tauri::State<AppState>) -> String {
    return match &state.parsed_shortcuts {
        Ok(parsed_shortcuts) => serde_json::to_string_pretty(parsed_shortcuts).unwrap(),
        Err(e) => return format!("{{ \"error\": \"{}\"}}", e),
    };
}

#[cfg(target_os = "linux")]
fn xdg_open_folder(folder_path: &str) -> io::Result<Child> {
    Command::new("xdg-open").arg(folder_path).spawn()
}

#[tauri::command]
fn open_appid_prefix(state: tauri::State<AppState>, userid: &str, appid: &str) {
    println!("Opening appid from rust! {}", appid);

    let Ok(parsed_shortcuts) = &state.parsed_shortcuts else {
        return;
    };
    let Some(userentries) = parsed_shortcuts.get(userid) else {
        return;
    };
    if !userentries.contains_key(appid) {
        return;
    }
    let Some(user_home_dir) = env::home_dir() else {
        return;
    };
    let steam_path = user_home_dir.join(".local/share/Steam");
    let prefix_path = steam_path.join(format!("steamapps/compatdata/{}/pfx/drive_c", appid));
    let Some(prefix_path_str) = prefix_path.to_str() else {
        return;
    };

    let _ = xdg_open_folder(prefix_path_str); // doesn't matter if error
}

fn get_all_steam_local_vdf_shortcuts() -> Result<HashMap<String, VdfMap>, String> {
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

    let mut result_map: HashMap<String, VdfMap> = user_ids
        .into_iter()
        .map(|userid| {
            let path = steam_path.join(format!("userdata/{}/config/shortcuts.vdf", &userid));
            (userid, path)
        })
        .filter(|(_, path)| path.is_file())
        .filter_map(|(userid, path)| {
            fs::read(&path)
                .ok()
                .and_then(|data| vdf_parser::read_vdf(data).ok())
                .map(|parsed| (userid, parsed))
        })
        .collect();

    // remove the "shortcuts" from the entry name
    let id_keys: Vec<String> = result_map.keys().map(|key| key.clone()).collect();
    for id in id_keys.iter() {
        let Some(mut popped) = result_map.remove(id) else {
            continue;
        };
        let targetobj = popped
            .remove("shortcuts")
            .ok_or("Unable to parse VDF".to_string())?;

        let targetobj = targetobj
            .into_map()
            .ok_or("Unable to parse VDF".to_string())?;

        result_map.insert(id.clone(), targetobj);
    }

    // Add appid as keys instead of indexes
    for (_, user_entries) in result_map.iter_mut() {
        let idx_keys: Vec<String> = user_entries.keys().map(|key| key.clone()).collect();
        for idx in idx_keys.iter() {
            let Some(entry) = user_entries.remove(idx) else {
                continue;
            };
            let Some(entryasmap) = entry.as_map() else {
                continue;
            };
            let Some(appid) = entryasmap.get("appid") else {
                continue;
            };
            let Some(appid) = appid.as_number() else {
                continue;
            };

            user_entries.insert(appid.to_string(), entry);
        }
    }

    // Now replace images for b64 content.
    for (_, user_entries) in result_map.iter_mut() {
        for (_, shortcutentry) in user_entries.iter_mut() {
            let Some(shortcutentry) = shortcutentry.as_map_mut() else {
                continue;
            };

            let Some(icon_val) = shortcutentry.get_mut("icon") else {
                continue;
            };

            let Some(icon_str) = icon_val.as_str() else {
                continue;
            };

            if icon_str.is_empty() {
                continue;
            }

            // load img from disk and display
            let Ok(image_bytes) = fs::read(icon_str) else {
                *icon_val = vdf_parser::VdfValue::String(String::from(""));
                continue;
            };

            let b64encoded = general_purpose::STANDARD.encode(&image_bytes);
            let mime = mime_from_extension(&icon_str);

            // replace
            *icon_val =
                vdf_parser::VdfValue::String(format!("data:{};base64,{}", mime, b64encoded));
        }
    }

    Ok(result_map)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let parsed_vdfs = get_all_steam_local_vdf_shortcuts();

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

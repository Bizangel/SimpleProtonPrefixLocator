// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod vdf_parser;

use std::collections::HashMap;
use std::env;
use std::env::home_dir;
use std::fs;
use std::io;
use std::path::PathBuf;
use tauri::{Builder, Manager};
use vdf_parser::VdfMap;

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

#[tauri::command]
fn open_appid_prefix(appid: &str) {
    println!("Opening appid from rust!");
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

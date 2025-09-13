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

// struct AppState {
//     parsed_shortcuts: Result<HashMap<String, VdfMap>, String>,
// }

#[tauri::command]
fn greet(name: &str) -> String {
    println!("Print from rust!");
    format!("Hello, {}! You've been greeted from Rust 2!", name)
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

    let result_map: HashMap<String, VdfMap> = user_ids
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

    Ok(result_map)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let foundres = get_all_steam_local_vdf_shortcuts();

    println!("{:#?}", foundres)
    // tauri::Builder::default()
    //     .setup(|app| {
    //         app.manage(AppState::default());
    //         Ok(())
    //     })
    //     .plugin(tauri_plugin_opener::init())
    //     .invoke_handler(tauri::generate_handler![greet])
    //     .run(tauri::generate_context!())
    //     .expect("error while running tauri application");
}

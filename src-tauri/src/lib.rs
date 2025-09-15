// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod binary_vdf_parser;
mod proton_app;
mod utils;
use binary_vdf_parser::VdfMap;
use proton_app::ProtonApp;
use std::fs;
use tauri::Manager;

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

#[tauri::command]
fn open_appid_prefix(state: tauri::State<AppState>, appid: &str) -> Result<(), String> {
    let parsed = state.parsed_shortcuts.as_ref()?;
    if !parsed.iter().any(|x| x.appid == appid) {
        return Err("Invalid AppID".to_string());
    }

    let steam_path = utils::get_steam_path()?;
    let prefix_path = steam_path.join(format!("steamapps/compatdata/{}/pfx/drive_c", appid));

    let _ = utils::xdg_open_folder(&prefix_path); // doesn't matter if error
    Ok(())
}

fn get_protonapps_from_vdf_shortcuts() -> Result<Vec<ProtonApp>, String> {
    let steam_path = utils::get_steam_path()?;
    let user_ids = utils::get_all_steam_user_ids(&steam_path)?;

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
        .filter_map(|x| ProtonApp::from_vdf_shortcut(&x).ok())
        .flatten()
        .collect();

    Ok(parsed)
}

// fn get_all_steam_apps() -> Result<Vec<ProtonApp>, String> {
//     let steam_path = utils::get_steam_path()?;
//     let user_ids = utils::get_all_steam_user_ids(&steam_path)?;
//     let vdfmaps: Vec<VdfMap> = user_ids
//         .iter()
//         .map(|userid| steam_path.join(format!("userdata/{}/config/shortcuts.vdf", &userid)))
//         .filter(|path| path.is_file())
//         .filter_map(|path| {
//             fs::read(&path)
//                 .ok()
//                 .and_then(|data| binary_vdf_parser::read_vdf(data).ok())
//         })
//         .collect();

//     let parsed: Vec<ProtonApp> = vdfmaps
//         .iter()
//         .filter_map(|x| ProtonApp::from_vdf_shortcut(&x).ok())
//         .flatten()
//         .collect();

//     Ok(parsed)
// }

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // let get_all_steam_apps = get_all_steam_apps();
    let shortcuts_apps = get_protonapps_from_vdf_shortcuts();

    tauri::Builder::default()
        .setup(|app| {
            app.manage(AppState {
                parsed_shortcuts: shortcuts_apps,
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

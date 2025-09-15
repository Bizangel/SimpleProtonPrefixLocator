// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod binary_vdf_parser;
mod proton_app;
mod utils;
use binary_vdf_parser::VdfMap;
use proton_app::ProtonApp;
use std::env;
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

    let user_home_dir = env::home_dir().ok_or_else(|| "Unable to fetch $HOME".to_string())?;
    let prefix_path = user_home_dir
        .join(".local/share/Steam")
        .join(format!("steamapps/compatdata/{}/pfx/drive_c", appid));

    let _ = utils::xdg_open_folder(&prefix_path); // doesn't matter if error
    Ok(())
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
        .filter_map(|x| ProtonApp::from_vdf_shortcut(&x).ok())
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

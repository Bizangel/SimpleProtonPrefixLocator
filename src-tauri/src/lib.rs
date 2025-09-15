// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod binary_vdf_parser;
mod proton_app;
mod utils;
use binary_vdf_parser::VdfMap;
use glob::glob;
use keyvalues_parser::Vdf as VdfText;
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

pub fn get_all_steam_apps() -> Result<Vec<ProtonApp>, String> {
    let steam_path = utils::get_steam_path()?;
    let steamapps_path = steam_path.join("steamapps");
    let pattern = format!("{}/appmanifest_*.acf", steamapps_path.display());

    /// Case-insensitive key lookup, returning first string or "" if missing
    fn get_str_ci<'a>(obj: &'a keyvalues_parser::Obj<'a>, key: &str) -> String {
        obj.iter()
            .find(|(k, _)| k.to_lowercase() == key.to_lowercase())
            .and_then(|(_, v)| v.first())
            .and_then(|v| v.get_str())
            .unwrap_or("")
            .to_string()
    }

    let apps: Vec<ProtonApp> = glob(&pattern)
        .map_err(|_| "Invalid glob pattern".to_string())?
        .filter_map(Result::ok)
        .filter_map(|manifest_path| {
            let text = fs::read_to_string(&manifest_path).ok()?;
            let parsed = VdfText::parse(&text).ok()?;
            let appstate = parsed.value.get_obj()?; // root is already AppState
            let appid = get_str_ci(appstate, "appid");
            let icon = utils::load_steam_app_icon(&steam_path, &appid).unwrap_or_default();

            Some(ProtonApp {
                appid,
                appname: get_str_ci(appstate, "name"),
                exe: get_str_ci(appstate, "exe"),
                startdir: get_str_ci(appstate, "installdir"),
                lastplaytime: get_str_ci(appstate, "lastplayed"),
                icon,
            })
        })
        .collect();

    Ok(apps)
}
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let all_steam_apps = get_all_steam_apps();
    let shortcuts_apps = get_protonapps_from_vdf_shortcuts();

    // merge both
    let merged =
        all_steam_apps.and_then(|v1| shortcuts_apps.map(|v2| v1.into_iter().chain(v2).collect()));

    tauri::Builder::default()
        .setup(|app| {
            app.manage(AppState {
                parsed_shortcuts: merged,
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

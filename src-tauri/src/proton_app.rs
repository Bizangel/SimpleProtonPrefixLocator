use crate::binary_vdf_parser::VdfMap;
use crate::utils;
use base64::{engine::general_purpose, Engine as _};
use serde::Serialize;
use std::path;

use std::fs;

#[derive(Debug, Serialize)]
pub struct ProtonApp {
    pub appname: String,
    pub appid: String,
    pub icon: String,
    pub lastplaytime: String,
    pub exe: String,
    pub startdir: String,
    pub installdir: String,
}

impl ProtonApp {
    pub fn from_vdf_shortcut(vdf: &VdfMap) -> Result<Vec<ProtonApp>, String> {
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
                            let icon_mime = utils::mime_from_extension(icon_path);
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
                    installdir: "".to_string(),
                })
            })
            .collect();

        Ok(apps)
    }

    pub fn prefix_path(&self) -> Option<path::PathBuf> {
        let steam_path = utils::get_steam_path().ok()?;
        let path = steam_path
            .join("steamapps/compatdata")
            .join(&self.appid)
            .join("pfx");
        Some(path)
    }

    pub fn prefix_path_exists(&self) -> bool {
        let prefix = match self.prefix_path() {
            Some(p) => p,
            None => return false,
        };

        prefix.is_dir()
            && prefix
                .parent()
                .map_or(false, |parent| parent.join("pfx.lock").is_file())
    }

    pub fn install_path(&self) -> Option<path::PathBuf> {
        let steam_path = utils::get_steam_path().ok()?;
        if self.installdir.is_empty() {
            return None;
        }

        Some(steam_path.join("steamapps/common").join(&self.installdir))
    }

    pub fn is_tool(&self) -> bool {
        let path = match self.install_path() {
            Some(p) => p,
            None => return false,
        };

        path.join("toolmanifest.vdf").is_file()
    }
}

use crate::binary_vdf_parser::VdfMap;
use crate::utils;
use base64::{engine::general_purpose, Engine as _};
use serde::Serialize;
use std::fs;

#[derive(Debug, Serialize)]
pub struct ProtonApp {
    pub appname: String,
    pub appid: String,
    pub icon: String,
    pub lastplaytime: String,
    pub exe: String,
    pub startdir: String,
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
                })
            })
            .collect();

        Ok(apps)
    }
}

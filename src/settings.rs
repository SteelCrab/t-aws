use crate::i18n::Language;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppSettings {
    pub language: Language,
}

fn get_settings_path() -> Option<PathBuf> {
    let home = dirs::home_dir()?;
    let emd_dir = home.join(".emd");

    if !emd_dir.exists() {
        fs::create_dir_all(&emd_dir).ok()?;
    }

    Some(emd_dir.join("settings.json"))
}

pub fn load_settings() -> AppSettings {
    let path = match get_settings_path() {
        Some(p) => p,
        None => return AppSettings::default(),
    };

    if !path.exists() {
        return AppSettings::default();
    }

    match fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => AppSettings::default(),
    }
}

pub fn save_settings(settings: &AppSettings) -> Result<(), std::io::Error> {
    let path = get_settings_path().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::NotFound, "Home directory not found")
    })?;

    let content = serde_json::to_string_pretty(settings)?;
    fs::write(&path, content)?;
    Ok(())
}

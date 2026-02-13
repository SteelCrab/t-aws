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

#[cfg(test)]
mod tests {
    use super::{AppSettings, load_settings, save_settings};
    use crate::i18n::Language;
    use std::env;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::{Mutex, OnceLock};

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    fn temp_home(prefix: &str) -> PathBuf {
        let mut path = env::temp_dir();
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or_default();
        path.push(format!("emd-{}-{}-{}", prefix, std::process::id(), nanos));
        path
    }

    struct HomeRestoreGuard {
        original_home: Option<std::ffi::OsString>,
        home: PathBuf,
    }

    impl HomeRestoreGuard {
        fn new(home: PathBuf) -> Self {
            let original_home = env::var_os("HOME");
            unsafe {
                env::set_var("HOME", &home);
            }
            Self {
                original_home,
                home,
            }
        }
    }

    impl Drop for HomeRestoreGuard {
        fn drop(&mut self) {
            if let Some(v) = self.original_home.as_ref() {
                unsafe {
                    env::set_var("HOME", v);
                }
            } else {
                unsafe {
                    env::remove_var("HOME");
                }
            }
            let _ = fs::remove_dir_all(&self.home);
        }
    }

    #[test]
    fn load_and_save_settings_round_trip_with_temp_home() {
        let _guard = env_lock().lock().expect("env lock poisoned");
        let home = temp_home("settings");
        fs::create_dir_all(&home).expect("create temp home");
        let _home_restore_guard = HomeRestoreGuard::new(home);

        let initial = load_settings();
        assert_eq!(initial.language, Language::English);

        let to_save = AppSettings {
            language: Language::Korean,
        };
        save_settings(&to_save).expect("save settings");

        let loaded = load_settings();
        assert_eq!(loaded.language, Language::Korean);
    }
}

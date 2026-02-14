#[path = "../src/env_bootstrap.rs"]
mod env_bootstrap;

use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

fn env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

struct EnvGuard {
    vars: Vec<(&'static str, Option<OsString>)>,
    cwd: PathBuf,
    temp_dir: Option<PathBuf>,
}

impl EnvGuard {
    fn new(keys: &[&'static str]) -> Self {
        let vars = keys
            .iter()
            .map(|k| (*k, std::env::var_os(k)))
            .collect::<Vec<_>>();
        let cwd = std::env::current_dir().expect("current dir");
        Self {
            vars,
            cwd,
            temp_dir: None,
        }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        let _ = std::env::set_current_dir(&self.cwd);
        for (key, value) in &self.vars {
            if let Some(v) = value {
                unsafe {
                    std::env::set_var(key, v);
                }
            } else {
                unsafe {
                    std::env::remove_var(key);
                }
            }
        }
        if let Some(temp_dir) = &self.temp_dir {
            let _ = fs::remove_dir_all(temp_dir);
        }
    }
}

fn make_temp_dir(prefix: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or_default();
    path.push(format!("emd-{}-{}-{}", prefix, std::process::id(), nanos));
    fs::create_dir_all(&path).expect("create temp dir");
    path
}

#[test]
fn load_dotenv_file_loads_aws_credentials_when_env_is_missing() {
    let _lock = env_lock().lock().expect("env lock");
    let mut guard = EnvGuard::new(&["AWS_ACCESS_KEY_ID", "AWS_SECRET_ACCESS_KEY"]);

    unsafe {
        std::env::remove_var("AWS_ACCESS_KEY_ID");
        std::env::remove_var("AWS_SECRET_ACCESS_KEY");
    }

    let temp = make_temp_dir("dotenv-load");
    guard.temp_dir = Some(temp.clone());
    fs::write(
        temp.join(".env"),
        "AWS_ACCESS_KEY_ID=test_key\nAWS_SECRET_ACCESS_KEY=test_secret\n",
    )
    .expect("write .env");
    std::env::set_current_dir(&temp).expect("set current dir");

    let loaded = env_bootstrap::load_dotenv_file();
    assert!(loaded.is_some());
    assert_eq!(
        std::env::var("AWS_ACCESS_KEY_ID").expect("AWS_ACCESS_KEY_ID"),
        "test_key"
    );
    assert_eq!(
        std::env::var("AWS_SECRET_ACCESS_KEY").expect("AWS_SECRET_ACCESS_KEY"),
        "test_secret"
    );
}

#[test]
fn load_dotenv_file_does_not_override_existing_env() {
    let _lock = env_lock().lock().expect("env lock");
    let mut guard = EnvGuard::new(&["AWS_ACCESS_KEY_ID", "AWS_SECRET_ACCESS_KEY"]);

    unsafe {
        std::env::set_var("AWS_ACCESS_KEY_ID", "existing_key");
        std::env::set_var("AWS_SECRET_ACCESS_KEY", "existing_secret");
    }

    let temp = make_temp_dir("dotenv-precedence");
    guard.temp_dir = Some(temp.clone());
    fs::write(
        temp.join(".env"),
        "AWS_ACCESS_KEY_ID=file_key\nAWS_SECRET_ACCESS_KEY=file_secret\n",
    )
    .expect("write .env");
    std::env::set_current_dir(&temp).expect("set current dir");

    let loaded = env_bootstrap::load_dotenv_file();
    assert!(loaded.is_some());
    assert_eq!(
        std::env::var("AWS_ACCESS_KEY_ID").expect("AWS_ACCESS_KEY_ID"),
        "existing_key"
    );
    assert_eq!(
        std::env::var("AWS_SECRET_ACCESS_KEY").expect("AWS_SECRET_ACCESS_KEY"),
        "existing_secret"
    );
}

#[test]
fn capture_aws_env_status_uses_defaults_when_not_configured() {
    let _lock = env_lock().lock().expect("env lock");
    let _guard = EnvGuard::new(&[
        "AWS_ACCESS_KEY_ID",
        "AWS_SECRET_ACCESS_KEY",
        "AWS_SESSION_TOKEN",
        "AWS_PROFILE",
        "AWS_DEFAULT_PROFILE",
        "AWS_REGION",
        "AWS_DEFAULT_REGION",
    ]);

    unsafe {
        std::env::remove_var("AWS_ACCESS_KEY_ID");
        std::env::remove_var("AWS_SECRET_ACCESS_KEY");
        std::env::remove_var("AWS_SESSION_TOKEN");
        std::env::remove_var("AWS_PROFILE");
        std::env::remove_var("AWS_DEFAULT_PROFILE");
        std::env::remove_var("AWS_REGION");
        std::env::remove_var("AWS_DEFAULT_REGION");
    }

    let status = env_bootstrap::capture_aws_env_status();
    assert!(!status.access_key_set);
    assert!(!status.secret_key_set);
    assert!(!status.session_token_set);
    assert_eq!(status.profile, "default");
    assert_eq!(status.region, "-");
}

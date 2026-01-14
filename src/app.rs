use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::download::DownloadProgress;
use crate::error::Result;
use crate::platform::Platform;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Install,
    Uninstall,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Screen {
    Welcome,
    Menu,
    Detecting,
    Confirm,
    Downloading,
    Installing,
    Complete,
    Uninstalling,
    UninstallComplete,
    Error,
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub screen: Screen,
    pub selected_action: Option<Action>,
    pub platform: Option<Platform>,
    pub download_progress: Option<DownloadProgress>,
    pub installer_path: Option<PathBuf>,
    pub aws_version: Option<String>,
    pub error_message: Option<String>,
    pub should_quit: bool,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            screen: Screen::Welcome,
            selected_action: None,
            platform: None,
            download_progress: None,
            installer_path: None,
            aws_version: None,
            error_message: None,
            should_quit: false,
        }
    }
}

pub struct App {
    pub state: Arc<Mutex<AppState>>,
}

impl App {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(AppState::default())),
        }
    }

    pub fn get_state(&self) -> AppState {
        self.state.lock().unwrap().clone()
    }

    pub fn set_screen(&self, screen: Screen) {
        self.state.lock().unwrap().screen = screen;
    }

    pub fn set_platform(&self, platform: Platform) {
        self.state.lock().unwrap().platform = Some(platform);
    }

    pub fn set_download_progress(&self, progress: DownloadProgress) {
        self.state.lock().unwrap().download_progress = Some(progress);
    }

    pub fn set_installer_path(&self, path: PathBuf) {
        self.state.lock().unwrap().installer_path = Some(path);
    }

    pub fn set_aws_version(&self, version: String) {
        self.state.lock().unwrap().aws_version = Some(version);
    }

    pub fn set_error(&self, message: String) {
        let mut state = self.state.lock().unwrap();
        state.error_message = Some(message);
        state.screen = Screen::Error;
    }

    pub fn set_action(&self, action: Action) {
        self.state.lock().unwrap().selected_action = Some(action);
    }

    pub fn quit(&self) {
        self.state.lock().unwrap().should_quit = true;
    }

    pub fn detect_platform(&self) -> Result<()> {
        let platform = Platform::detect()?;
        self.set_platform(platform);
        self.set_screen(Screen::Confirm);
        Ok(())
    }
}

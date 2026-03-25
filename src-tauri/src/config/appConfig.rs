use std::fs;
use std::{default, path::PathBuf};

use tauri::{App, AppHandle, Manager};

#[derive(Debug)]
pub struct AppConfig {
    pub use_custom_dir: bool,
    pub novel_path: PathBuf,
    pub mp3_path: PathBuf,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            use_custom_dir: false,
            novel_path: PathBuf::new(),
            mp3_path: PathBuf::new(),
        }
    }
}

pub fn load_config(app: &AppHandle) -> AppConfig {
    let conf_dir = app.path().config_dir().expect("cant load config from dir");

    todo!()
}

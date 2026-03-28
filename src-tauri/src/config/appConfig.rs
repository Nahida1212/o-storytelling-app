use std::{clone, fs};
use std::{default, path::PathBuf};

use serde::{Deserialize, Serialize};
use tauri::{App, AppHandle, Manager};
use tauri_ts_generator::TS;

#[derive(Debug,Clone,Serialize,Deserialize,TS)]
#[serde(rename_all = "camelCase")]
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

pub fn get_config(app: &AppHandle) -> AppConfig {
    let conf_dir = app.path().config_dir().expect("cant load config from dir");

    todo!()
}

use std::{default, sync::Mutex};

use tauri::AppHandle;

use crate::{
    config::{
        self,
        appConfig::{self, load_config, AppConfig},
    },
    state::appState,
};

#[derive(Debug)]
pub struct AppState {
    config: Mutex<AppConfig>,
}
impl Default for AppState {
    fn default() -> Self {
        Self {
            config: Mutex::new(AppConfig::default()),
        }
    }
}

pub fn load_state() -> Result<String, String> {
    // read from disk

    let mut app_state = AppState::default();

    Ok("fsfs".to_owned())
}

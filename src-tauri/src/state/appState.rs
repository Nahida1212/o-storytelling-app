use tauri::State;

use crate::{
    config::{
        self,
        appConfig::{self, get_config, AppConfig},
    },
    state::{self, appState},
};
use std::sync::Mutex;

#[derive(Debug)]
pub struct AppState {
    pub config: Mutex<AppConfig>,
}
impl Default for AppState {
    fn default() -> Self {
        Self {
            config: Mutex::new(AppConfig::default()),
        }
    }
}

pub fn load_state() -> Result<AppState, String> {
    // read from disk

    // fisrt read return defule
    Ok(AppState::default())
}

fn get_state() {}

#[tauri::command]
pub fn get_config_state(state: State<AppState>) -> AppConfig {
    state.config.lock().unwrap().clone()
}

#[tauri::command]
pub fn updata_config_state(state: State<AppState>, config: AppConfig) -> Result<String, String> {
    let mut current_config = state.config.lock().map_err(|e| e.to_string())?;
    *current_config = config;

    Ok("config ok".to_owned())
}

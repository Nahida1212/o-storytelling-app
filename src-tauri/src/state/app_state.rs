use tauri::{AppHandle, State};

use crate::config::app_config::{self, AppConfig};
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
    Ok(AppState::default())
}

/// 从磁盘加载配置并更新 state（在 app setup 中调用）
pub fn load_config_into_state(app: &AppHandle, state: &AppState) {
    if let Ok(config) = app_config::load_config(app) {
        if let Ok(mut current) = state.config.lock() {
            *current = config;
        }
    }
}

#[tauri::command]
pub fn get_config_state(state: State<AppState>) -> AppConfig {
    state.config.lock().unwrap().clone()
}

#[tauri::command]
pub fn updata_config_state(
    app: AppHandle,
    state: State<AppState>,
    config: AppConfig,
) -> Result<String, String> {
    let mut current_config = state.config.lock().map_err(|e| e.to_string())?;
    *current_config = config.clone();

    // 持久化到 resource_dir()/setting.json
    app_config::save_config(&app, &config)?;

    Ok("config ok".to_owned())
}

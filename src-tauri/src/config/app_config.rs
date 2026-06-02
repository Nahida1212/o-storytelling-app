use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use tauri_ts_generator::TS;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub use_custom_dir: bool,
    pub novel_path: PathBuf,
    pub mp3_path: PathBuf,
    pub image_path: PathBuf,
    pub gpt_sovits_path: PathBuf,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            use_custom_dir: false,
            novel_path: PathBuf::new(),
            mp3_path: PathBuf::new(),
            image_path: PathBuf::new(),
            gpt_sovits_path: PathBuf::new(),
        }
    }
}

pub fn get_config_path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app.path().resource_dir().map_err(|e| e.to_string())?;
    Ok(dir.join("setting.json"))
}

pub fn save_config(app: &AppHandle, config: &AppConfig) -> Result<(), String> {
    let path = get_config_path(app)?;
    let json = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    std::fs::write(&path, &json).map_err(|e| format!("保存配置文件失败: {}", e))
}

pub fn load_config(app: &AppHandle) -> Result<AppConfig, String> {
    let path = get_config_path(app)?;
    let json = std::fs::read_to_string(&path).map_err(|e| format!("读取配置文件失败: {}", e))?;
    serde_json::from_str(&json).map_err(|e| format!("解析配置文件失败: {}", e))
}

pub fn get_config(app: &AppHandle) -> AppConfig {
    let _conf_dir = app.path().config_dir().expect("cant load config from dir");

    todo!()
}

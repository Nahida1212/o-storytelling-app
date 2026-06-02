use std::collections::HashMap;
use std::fs;
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;

use tauri::{AppHandle, Manager, State};

use crate::db::db_service;
use crate::state::app_state::AppState;

pub struct EngineProcesses(pub Mutex<HashMap<i64, Child>>);

impl EngineProcesses {
    pub fn new() -> Self {
        Self(Mutex::new(HashMap::new()))
    }
}

impl Drop for EngineProcesses {
    fn drop(&mut self) {
        if let Ok(mut map) = self.0.lock() {
            for (_, mut child) in map.drain() {
                let _ = child.kill();
                let _ = child.wait();
            }
        }
    }
}

fn parse_port(api_base_url: &str) -> &str {
    let base = api_base_url.trim_end_matches('/');
    if let Some(port_start) = base.rfind(':') {
        let after_colon = &base[port_start + 1..];
        if after_colon
            .chars()
            .next()
            .map_or(false, |c| c.is_ascii_digit())
        {
            let port = after_colon.split(['/', '?']).next().unwrap_or(after_colon);
            return port;
        }
    }
    "9880"
}

#[tauri::command]
pub fn start_engine(
    app: AppHandle,
    processes: State<EngineProcesses>,
    character_id: i64,
) -> Result<(), String> {
    let voice = db_service::get_character_voice_by_id(&app, character_id)
        .map_err(|e| format!("获取角色信息失败: {}", e))?;

    let config_path = voice
        .config_path
        .as_deref()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| "角色没有配置文件路径，请先保存角色")?
        .to_string();

    let app_state = app.state::<AppState>();
    let config = app_state.config.lock().map_err(|e| e.to_string())?;
    let engine_dir = config.gpt_sovits_path.clone();
    drop(config);

    if engine_dir.as_os_str().is_empty() {
        return Err("请先在设置中配置 GPT-SoVITS 引擎目录".to_string());
    }

    let python = engine_dir.join("runtime").join("python.exe");
    if !python.exists() {
        return Err("未找到 Python 运行时，请检查引擎目录".to_string());
    }

    let api_script = engine_dir.join("api_v2.py");
    if !api_script.exists() {
        return Err("未找到 api_v2.py，请检查引擎目录".to_string());
    }

    let port = parse_port(&voice.api_base_url);

    // 停止旧进程
    let mut map = processes.0.lock().map_err(|e| e.to_string())?;
    if let Some(mut old) = map.remove(&character_id) {
        let _ = old.kill();
        let _ = old.wait();
    }

    // 将 stderr 写入日志文件以便排查
    let log_dir = app
        .path()
        .resource_dir()
        .map_err(|e| e.to_string())?
        .join("logs");
    let _ = fs::create_dir_all(&log_dir);
    let stderr_path = log_dir.join(format!("engine_{}.log", character_id));
    let stderr_file =
        fs::File::create(&stderr_path).map_err(|e| format!("创建日志文件失败: {}", e))?;

    let child = Command::new(&python)
        .arg(&api_script)
        .arg("-a")
        .arg("127.0.0.1")
        .arg("-p")
        .arg(port)
        .arg("-c")
        .arg(&config_path)
        .current_dir(&engine_dir)
        .stdout(Stdio::null())
        .stderr(stderr_file)
        .spawn()
        .map_err(|e| format!("启动引擎失败: {}", e))?;

    map.insert(character_id, child);
    Ok(())
}

#[tauri::command]
pub fn stop_engine(processes: State<EngineProcesses>, character_id: i64) -> Result<(), String> {
    let mut map = processes.0.lock().map_err(|e| e.to_string())?;
    if let Some(mut child) = map.remove(&character_id) {
        child.kill().map_err(|e| format!("停止引擎失败: {}", e))?;
        let _ = child.wait();
    }
    Ok(())
}

#[tauri::command]
pub fn get_engine_process_status(
    processes: State<EngineProcesses>,
    character_id: i64,
) -> Result<String, String> {
    let mut map = processes.0.lock().map_err(|e| e.to_string())?;
    match map.get_mut(&character_id) {
        Some(child) => match child.try_wait() {
            Ok(Some(_)) => Ok("stopped".to_string()),
            Ok(None) => Ok("running".to_string()),
            Err(_) => Ok("error".to_string()),
        },
        None => Ok("stopped".to_string()),
    }
}

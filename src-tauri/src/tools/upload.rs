use std::{
    fs::{self, create_dir},
    intrinsics::copy,
    path::{Path, PathBuf},
};

use crate::state::appState::AppState;
use rusqlite::ffi::SQLITE_CREATE_INDEX;
use tauri::{AppHandle, Error, Manager, State};
use tauri_ts_generator::config;
use tauri_plugin_fs::FsExt;

#[tauri::command]
pub fn file_upload(state: State<AppState>, file: String) -> Result<String, String> {
    println!("file {}", file);

    let mut book_path = PathBuf::new();
    book_path.push(file);
    println!("filepath {}", book_path.to_string_lossy());

    let mut file_name = book_path.file_name().unwrap();

    let config_state = state.config.lock().unwrap().clone();

    
    println!("config paht {}", config_state.novel_path.to_string_lossy());
    let mut to_path = config_state.novel_path.clone();
    to_path.push(file_name);
    println!("to_path {}" , to_path.to_string_lossy());
    

    // copy file
    if config_state.use_custom_dir {
        // use  custom dir
        if config_state.novel_path.exists() {
            // path no null, copy file to path where setting
            fs::copy(&book_path, to_path)
                .map_err(|e: std::io::Error| e.to_string())?;

            // processing the novel file

            // insert the sql
        } else {
            // no set dir
            // use file original path

            println!("bucuizai")
        }
    }

    // fs::copy(&book_path, &save_path);
    Ok("upload file success".to_owned())
}

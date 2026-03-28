use tauri::AppHandle;

use crate::state::appState::{self, AppState};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}
pub mod config;
pub mod state;
pub mod tools;

#[cfg_attr(mobile, tauri::mobile_entry_point)]

pub fn run() {
    let app_state = state::appState::load_state().unwrap_or_else(|err| {
        println!("load fail");
        AppState::default()
    });

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            greet,
            tools::upload::file_upload,
            state::appState::get_config_state,
            state::appState::updata_config_state
        ])
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn test_lsp() {
    let x: Option<i32> = Some(10);
    x.unwrap();
}

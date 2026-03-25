use tauri::AppHandle;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}
mod config;
mod state;
mod tools;

#[cfg_attr(mobile, tauri::mobile_entry_point)]

pub fn run() {
    let app_state = state::appState::load_state();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![greet, tools::upload::file_upload])
        .plugin(tauri_plugin_dialog::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn test_lsp() {
    let x: Option<i32> = Some(10);
    x.unwrap();
}

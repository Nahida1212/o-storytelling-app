use crate::state::appState::AppState;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}
pub mod config;
pub mod state;
pub mod tools;
pub mod db;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_state = state::appState::load_state().unwrap_or_else(|_err| {
        println!("load fail");
        AppState::default()
    });

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // 初始化数据库
            db::dbStart::initialize_database(app.handle()).expect("数据库初始化失败");
            Ok(())
        })
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            greet,
            tools::upload::file_upload,
            tools::upload::get_all_books,
            tools::illustrations::get_illustrations_by_novel,
            tools::illustrations::add_illustration,
            tools::illustrations::delete_illustration,
            tools::illustrations::update_illustration_description,
            tools::upload::get_book_details,
            tools::upload::get_book_chapters,
            tools::upload::delete_books,
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

use std::sync::Mutex;
use crate::state::appState::AppState;
use tauri::Manager;

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
        .manage(app_state)
        .manage(tools::engine_manager::EngineProcesses::new())
        .setup(|app| {
            // 初始化数据库
            db::dbStart::initialize_database(app.handle()).expect("数据库初始化失败");

            // 从磁盘加载配置到 state
            let state = app.state::<AppState>();
            state::appState::load_config_into_state(app.handle(), state.inner());

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            tools::upload::get_all_books,
            tools::illustrations::get_illustrations_by_novel,
            tools::illustrations::add_illustration,
            tools::illustrations::delete_illustration,
            tools::illustrations::update_illustration_description,
            tools::upload::get_book_details,
            tools::upload::get_book_chapters,
            tools::upload::delete_books,
            tools::upload::import_book,
            tools::upload::get_chapter_content,
            tools::chapter_processor::process_chapters,
            tools::chapter_processor::call_llm_on_chunk,
            tools::chapter_processor::start_tts_generation,
            tools::chapter_processor::get_tts_generated_chapters,
            tools::chapter_processor::get_chapter_tts_summary,
            tools::chapter_processor::get_all_chapters,
            tools::chapter_processor::get_chapter_character_mappings,
            tools::chapter_processor::save_chapter_character_mappings,
            tools::chapter_processor::generate_chapter_audio,
            tools::character_voice::get_all_character_voices,
            tools::character_voice::create_character_voice,
            tools::character_voice::update_character_voice,
            tools::character_voice::delete_character_voice,
            tools::engine_manager::start_engine,
            tools::engine_manager::stop_engine,
            tools::engine_manager::get_engine_process_status,
            tools::audio_player::get_novels_with_audio,
            tools::audio_player::get_chapters_with_audio,
            tools::audio_player::get_chapter_scenes_with_audio,
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

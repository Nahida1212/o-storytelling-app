use tauri::AppHandle;

use crate::db::db_service;

/// 获取所有有已生成音频的小说
#[tauri::command]
pub fn get_novels_with_audio(app: AppHandle) -> Result<Vec<db_service::NovelWithAudioInfo>, String> {
    db_service::get_novels_with_audio(&app).map_err(|e| format!("获取有声小说列表失败: {}", e))
}

/// 获取指定小说下有已生成音频的章节
#[tauri::command]
pub fn get_chapters_with_audio(
    app: AppHandle,
    novel_id: i64,
) -> Result<Vec<db_service::ChapterWithAudioInfo>, String> {
    db_service::get_chapters_with_audio(&app, novel_id).map_err(|e| format!("获取有声章节列表失败: {}", e))
}

/// 获取指定章节所有有音频的场景（用于歌词展示和播放）
#[tauri::command]
pub fn get_chapter_scenes_with_audio(
    app: AppHandle,
    chapter_id: i64,
) -> Result<Vec<db_service::TtsSceneWithAudio>, String> {
    db_service::get_tts_scenes_with_audio_by_chapter(&app, chapter_id)
        .map_err(|e| format!("获取场景音频列表失败: {}", e))
}

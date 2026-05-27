use tauri::AppHandle;

use crate::db::dbService;

/// 获取所有有已生成音频的小说
#[tauri::command]
pub fn get_novels_with_audio(app: AppHandle) -> Result<Vec<dbService::NovelWithAudioInfo>, String> {
    dbService::get_novels_with_audio(&app).map_err(|e| format!("获取有声小说列表失败: {}", e))
}

/// 获取指定小说下有已生成音频的章节
#[tauri::command]
pub fn get_chapters_with_audio(
    app: AppHandle,
    novel_id: i64,
) -> Result<Vec<dbService::ChapterWithAudioInfo>, String> {
    dbService::get_chapters_with_audio(&app, novel_id).map_err(|e| format!("获取有声章节列表失败: {}", e))
}

/// 获取指定章节所有有音频的场景（用于歌词展示和播放）
#[tauri::command]
pub fn get_chapter_scenes_with_audio(
    app: AppHandle,
    chapter_id: i64,
) -> Result<Vec<dbService::TtsSceneWithAudio>, String> {
    dbService::get_tts_scenes_with_audio_by_chapter(&app, chapter_id)
        .map_err(|e| format!("获取场景音频列表失败: {}", e))
}

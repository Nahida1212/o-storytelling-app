use tauri::AppHandle;
use crate::db::dbService;

/// 插图信息（用于前端显示）
#[derive(serde::Serialize, tauri_ts_generator::TS)]
#[ts(export)]
pub struct IllustrationInfo {
    pub id: i64,
    pub novel_id: i64,
    pub image_path: String,
    pub description: Option<String>,
    pub chapter_index: Option<i32>,
}

/// 获取小说的所有插图
#[tauri::command]
pub fn get_illustrations_by_novel(app: AppHandle, novel_id: i64) -> Result<Vec<IllustrationInfo>, String> {
    let illustrations = dbService::get_illustrations_by_novel_id(&app, novel_id)
        .map_err(|e| format!("Failed to get illustrations: {}", e))?;

    let illustration_infos: Vec<IllustrationInfo> = illustrations
        .into_iter()
        .map(|illustration| {
            IllustrationInfo {
                id: illustration.id.unwrap_or(0),
                novel_id: illustration.novel_id,
                image_path: illustration.image_path,
                description: illustration.description,
                chapter_index: illustration.chapter_index,
            }
        })
        .collect();

    Ok(illustration_infos)
}

/// 添加插图
#[tauri::command]
pub fn add_illustration(
    app: AppHandle,
    novel_id: i64,
    image_path: String,
    description: Option<String>,
    chapter_index: Option<i32>,
) -> Result<i64, String> {
    let illustration_data = dbService::IllustrationData {
        id: None,
        novel_id,
        image_path,
        description,
        chapter_index,
    };

    let illustration_id = dbService::insert_illustration(&app, &illustration_data)
        .map_err(|e| format!("Failed to insert illustration: {}", e))?;

    Ok(illustration_id)
}

/// 删除插图
#[tauri::command]
pub fn delete_illustration(app: AppHandle, illustration_id: i64) -> Result<(), String> {
    dbService::delete_illustration(&app, illustration_id)
        .map_err(|e| format!("Failed to delete illustration: {}", e))?;

    Ok(())
}

/// 更新插图描述
#[tauri::command]
pub fn update_illustration_description(
    app: AppHandle,
    illustration_id: i64,
    description: Option<String>,
) -> Result<(), String> {
    dbService::update_illustration_description(&app, illustration_id, description.as_deref())
        .map_err(|e| format!("Failed to update illustration description: {}", e))?;

    Ok(())
}
use rusqlite::{params, Result};
use std::path::{Path, PathBuf};

use super::db_start;

/// 章节完整信息：(chapter_index, title, content, novel_id, chapter_id)
type ChapterFullInfo = (i32, String, String, i64, i64);

/// 小说数据（用于数据库插入和查询）
#[derive(Debug)]
pub struct NovelData {
    pub id: Option<i64>, // 插入时为None，查询时为Some
    pub title: String,
    pub author: Option<String>,
    pub file_path: PathBuf,
    pub cover_image: Option<Vec<u8>>,
    pub cover_image_path: Option<String>,
}

/// 章节数据（用于数据库插入）
#[derive(Debug)]
pub struct ChapterData {
    pub novel_id: i64,
    pub chapter_index: i32,
    pub title: String,
    pub content: Option<String>,
    pub audio_path: Option<String>,
    pub tts_generated: bool,
}

/// 插入小说到数据库
/// 返回新插入小说的ID
pub fn insert_novel(app_handle: &tauri::AppHandle, novel_data: &NovelData) -> Result<i64> {
    let conn = db_start::get_database_connection(app_handle)?;

    conn.execute(
        "INSERT INTO novels (title, author, file_path, cover_image, cover_image_path, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
        params![
            novel_data.title,
            novel_data.author,
            novel_data.file_path.to_string_lossy(),
            novel_data.cover_image,
            novel_data.cover_image_path,
        ],
    )?;

    Ok(conn.last_insert_rowid())
}

/// 插入章节到数据库
pub fn insert_chapter(app_handle: &tauri::AppHandle, chapter_data: &ChapterData) -> Result<()> {
    let conn = db_start::get_database_connection(app_handle)?;

    conn.execute(
        "INSERT INTO chapters (novel_id, chapter_index, title, content, audio_path, tts_generated, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
        params![
            chapter_data.novel_id,
            chapter_data.chapter_index,
            chapter_data.title,
            chapter_data.content,
            chapter_data.audio_path,
            chapter_data.tts_generated as i32,
        ],
    )?;

    Ok(())
}

/// 批量插入章节
pub fn insert_chapters_batch(
    app_handle: &tauri::AppHandle,
    chapters: &[ChapterData],
) -> Result<()> {
    let mut conn = db_start::get_database_connection(app_handle)?;

    // 开始事务
    let tx = conn.transaction()?;

    for chapter in chapters {
        tx.execute(
            "INSERT INTO chapters (novel_id, chapter_index, title, content, audio_path, tts_generated, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
            params![
                chapter.novel_id,
                chapter.chapter_index,
                chapter.title,
                chapter.content,
                chapter.audio_path,
                chapter.tts_generated as i32,
            ],
        )?;
    }

    tx.commit()?;
    Ok(())
}

/// 根据文件路径查找小说ID
pub fn find_novel_by_file_path(
    app_handle: &tauri::AppHandle,
    file_path: &Path,
) -> Result<Option<i64>> {
    let conn = db_start::get_database_connection(app_handle)?;

    let mut stmt = conn.prepare("SELECT id FROM novels WHERE file_path = ?1")?;
    let mut rows = stmt.query(params![file_path.to_string_lossy()])?;

    if let Some(row) = rows.next()? {
        Ok(Some(row.get(0)?))
    } else {
        Ok(None)
    }
}

/// 获取小说所有章节
pub fn get_chapters_by_novel_id(
    app_handle: &tauri::AppHandle,
    novel_id: i64,
) -> Result<Vec<ChapterData>> {
    let conn = db_start::get_database_connection(app_handle)?;

    let mut stmt = conn.prepare(
        "SELECT novel_id, chapter_index, title, content, audio_path, tts_generated
         FROM chapters WHERE novel_id = ?1 ORDER BY chapter_index",
    )?;

    let rows = stmt.query_map(params![novel_id], |row| {
        Ok(ChapterData {
            novel_id: row.get(0)?,
            chapter_index: row.get(1)?,
            title: row.get(2)?,
            content: row.get(3)?,
            audio_path: row.get(4)?,
            tts_generated: row.get::<_, i32>(5)? != 0,
        })
    })?;

    let mut chapters = Vec::new();
    for row in rows {
        chapters.push(row?);
    }

    Ok(chapters)
}

/// 获取单章内容（根据 novel_id 和 chapter_index）
pub fn get_chapter_by_index(
    app_handle: &tauri::AppHandle,
    novel_id: i64,
    chapter_index: i32,
) -> Result<Option<ChapterData>> {
    let conn = db_start::get_database_connection(app_handle)?;

    let mut stmt = conn.prepare(
        "SELECT novel_id, chapter_index, title, content, audio_path, tts_generated
         FROM chapters WHERE novel_id = ?1 AND chapter_index = ?2",
    )?;

    let mut rows = stmt.query_map(params![novel_id, chapter_index], |row| {
        Ok(ChapterData {
            novel_id: row.get(0)?,
            chapter_index: row.get(1)?,
            title: row.get(2)?,
            content: row.get(3)?,
            audio_path: row.get(4)?,
            tts_generated: row.get::<_, i32>(5)? != 0,
        })
    })?;

    match rows.next() {
        Some(row) => Ok(Some(row?)),
        None => Ok(None),
    }
}

/// 更新章节的 TTS 生成状态
pub fn update_chapter_tts_generated(
    app_handle: &tauri::AppHandle,
    chapter_id: i64,
    generated: bool,
) -> Result<()> {
    let conn = db_start::get_database_connection(app_handle)?;

    conn.execute(
        "UPDATE chapters SET tts_generated = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = ?2",
        params![generated as i32, chapter_id],
    )?;

    Ok(())
}

/// 批量更新章节的 TTS 生成状态（根据 novel_id + chapter_index）
pub fn update_chapters_tts_generated_by_indices(
    app_handle: &tauri::AppHandle,
    novel_id: i64,
    chapter_indices: &[i32],
    generated: bool,
) -> Result<()> {
    let conn = db_start::get_database_connection(app_handle)?;

    if chapter_indices.is_empty() {
        return Ok(());
    }

    let placeholders: Vec<String> = chapter_indices.iter().map(|_| "?".to_string()).collect();
    let sql = format!(
        "UPDATE chapters SET tts_generated = ?1, updated_at = CURRENT_TIMESTAMP \
         WHERE novel_id = ?2 AND chapter_index IN ({})",
        placeholders.join(",")
    );

    let mut stmt = conn.prepare(&sql)?;

    let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
    param_values.push(Box::new(generated as i32));
    param_values.push(Box::new(novel_id));
    for idx in chapter_indices {
        param_values.push(Box::new(*idx));
    }

    let params_refs: Vec<&dyn rusqlite::types::ToSql> =
        param_values.iter().map(|p| p.as_ref()).collect();
    stmt.execute(params_refs.as_slice())?;

    Ok(())
}

/// 删除小说及其所有章节
pub fn delete_novel(app_handle: &tauri::AppHandle, novel_id: i64) -> Result<()> {
    let conn = db_start::get_database_connection(app_handle)?;

    // 由于外键设置了 ON DELETE CASCADE，删除小说会自动删除相关章节
    conn.execute("DELETE FROM novels WHERE id = ?1", params![novel_id])?;

    Ok(())
}

/// 更新小说信息
pub fn update_novel(
    app_handle: &tauri::AppHandle,
    novel_id: i64,
    title: &str,
    author: Option<&str>,
) -> Result<()> {
    let conn = db_start::get_database_connection(app_handle)?;

    conn.execute(
        "UPDATE novels SET title = ?1, author = ?2, updated_at = CURRENT_TIMESTAMP WHERE id = ?3",
        params![title, author, novel_id],
    )?;

    Ok(())
}

/// 获取所有小说
pub fn get_all_novels(app_handle: &tauri::AppHandle) -> Result<Vec<NovelData>> {
    let conn = db_start::get_database_connection(app_handle)?;

    let mut stmt = conn.prepare(
        "SELECT id, title, author, file_path, cover_image, cover_image_path FROM novels ORDER BY created_at DESC"
    )?;

    let rows = stmt.query_map([], |row| {
        Ok(NovelData {
            id: Some(row.get(0)?),
            title: row.get(1)?,
            author: row.get(2)?,
            file_path: PathBuf::from(row.get::<_, String>(3)?),
            cover_image: row.get(4)?,
            cover_image_path: row.get(5)?,
        })
    })?;

    let mut novels = Vec::new();
    for row in rows {
        novels.push(row?);
    }

    Ok(novels)
}

/// 根据ID获取小说
pub fn get_novel_by_id(app_handle: &tauri::AppHandle, novel_id: i64) -> Result<Option<NovelData>> {
    let conn = db_start::get_database_connection(app_handle)?;

    let mut stmt = conn.prepare(
        "SELECT id, title, author, file_path, cover_image, cover_image_path FROM novels WHERE id = ?1"
    )?;

    let mut rows = stmt.query_map(params![novel_id], |row| {
        Ok(NovelData {
            id: Some(row.get(0)?),
            title: row.get(1)?,
            author: row.get(2)?,
            file_path: PathBuf::from(row.get::<_, String>(3)?),
            cover_image: row.get(4)?,
            cover_image_path: row.get(5)?,
        })
    })?;

    if let Some(row) = rows.next() {
        Ok(Some(row?))
    } else {
        Ok(None)
    }
}

/// 插图数据（用于数据库插入和查询）
#[derive(Debug)]
pub struct IllustrationData {
    pub id: Option<i64>, // 插入时为None，查询时为Some
    pub novel_id: i64,
    pub image_path: String,
    pub description: Option<String>,
    pub chapter_index: Option<i32>, // 关联的章节索引（可为空）
}

/// 插入插图到数据库
/// 返回新插入插图的ID
pub fn insert_illustration(
    app_handle: &tauri::AppHandle,
    illustration_data: &IllustrationData,
) -> Result<i64> {
    let conn = db_start::get_database_connection(app_handle)?;

    conn.execute(
        "INSERT INTO illustrations (novel_id, image_path, description, chapter_index, created_at)
         VALUES (?1, ?2, ?3, ?4, CURRENT_TIMESTAMP)",
        params![
            illustration_data.novel_id,
            illustration_data.image_path,
            illustration_data.description,
            illustration_data.chapter_index,
        ],
    )?;

    Ok(conn.last_insert_rowid())
}

/// 批量插入插图
pub fn insert_illustrations_batch(
    app_handle: &tauri::AppHandle,
    illustrations: &[IllustrationData],
) -> Result<()> {
    let mut conn = db_start::get_database_connection(app_handle)?;

    // 开始事务
    let tx = conn.transaction()?;

    for illustration in illustrations {
        tx.execute(
            "INSERT INTO illustrations (novel_id, image_path, description, chapter_index, created_at)
             VALUES (?1, ?2, ?3, ?4, CURRENT_TIMESTAMP)",
            params![
                illustration.novel_id,
                illustration.image_path,
                illustration.description,
                illustration.chapter_index,
            ],
        )?;
    }

    tx.commit()?;
    Ok(())
}

/// 根据小说ID获取所有插图
pub fn get_illustrations_by_novel_id(
    app_handle: &tauri::AppHandle,
    novel_id: i64,
) -> Result<Vec<IllustrationData>> {
    let conn = db_start::get_database_connection(app_handle)?;

    let mut stmt = conn.prepare(
        "SELECT id, novel_id, image_path, description, chapter_index
         FROM illustrations WHERE novel_id = ?1 ORDER BY created_at",
    )?;

    let rows = stmt.query_map(params![novel_id], |row| {
        Ok(IllustrationData {
            id: Some(row.get(0)?),
            novel_id: row.get(1)?,
            image_path: row.get(2)?,
            description: row.get(3)?,
            chapter_index: row.get(4)?,
        })
    })?;

    let mut illustrations = Vec::new();
    for row in rows {
        illustrations.push(row?);
    }

    Ok(illustrations)
}

/// 删除插图
pub fn delete_illustration(app_handle: &tauri::AppHandle, illustration_id: i64) -> Result<()> {
    let conn = db_start::get_database_connection(app_handle)?;

    conn.execute(
        "DELETE FROM illustrations WHERE id = ?1",
        params![illustration_id],
    )?;

    Ok(())
}

/// 根据小说ID删除所有插图
pub fn delete_illustrations_by_novel_id(
    app_handle: &tauri::AppHandle,
    novel_id: i64,
) -> Result<()> {
    let conn = db_start::get_database_connection(app_handle)?;

    conn.execute(
        "DELETE FROM illustrations WHERE novel_id = ?1",
        params![novel_id],
    )?;

    Ok(())
}

/// 更新插图描述
pub fn update_illustration_description(
    app_handle: &tauri::AppHandle,
    illustration_id: i64,
    description: Option<&str>,
) -> Result<()> {
    let conn = db_start::get_database_connection(app_handle)?;

    conn.execute(
        "UPDATE illustrations SET description = ?1 WHERE id = ?2",
        params![description, illustration_id],
    )?;

    Ok(())
}

/// 更新小说封面图片路径
pub fn update_novel_cover_image_path(
    app_handle: &tauri::AppHandle,
    novel_id: i64,
    cover_image_path: Option<&str>,
) -> Result<()> {
    let conn = db_start::get_database_connection(app_handle)?;

    conn.execute(
        "UPDATE novels SET cover_image_path = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = ?2",
        params![cover_image_path, novel_id],
    )?;

    Ok(())
}

/// 角色对话数据（用于数据库插入和查询）
#[derive(Debug)]
pub struct DialogueData {
    pub id: Option<i64>, // 插入时为None，查询时为Some
    pub novel_id: i64,
    pub chapter_index: i32,
    pub dialogue_index: i32,
    pub character_name: String,
    pub content: String,
    pub audio_path: Option<String>,
}

/// 插入角色对话
/// 返回新插入对话的ID
pub fn insert_dialogue(app_handle: &tauri::AppHandle, dialogue_data: &DialogueData) -> Result<i64> {
    let conn = db_start::get_database_connection(app_handle)?;

    conn.execute(
        "INSERT INTO dialogues (novel_id, chapter_index, dialogue_index, character_name, content, audio_path, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, CURRENT_TIMESTAMP)",
        params![
            dialogue_data.novel_id,
            dialogue_data.chapter_index,
            dialogue_data.dialogue_index,
            dialogue_data.character_name,
            dialogue_data.content,
            dialogue_data.audio_path,
        ],
    )?;

    Ok(conn.last_insert_rowid())
}

/// 批量插入角色对话
pub fn insert_dialogues_batch(
    app_handle: &tauri::AppHandle,
    dialogues: &[DialogueData],
) -> Result<()> {
    let mut conn = db_start::get_database_connection(app_handle)?;

    let tx = conn.transaction()?;

    for dialogue in dialogues {
        tx.execute(
            "INSERT INTO dialogues (novel_id, chapter_index, dialogue_index, character_name, content, audio_path, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, CURRENT_TIMESTAMP)",
            params![
                dialogue.novel_id,
                dialogue.chapter_index,
                dialogue.dialogue_index,
                dialogue.character_name,
                dialogue.content,
                dialogue.audio_path,
            ],
        )?;
    }

    tx.commit()?;
    Ok(())
}

/// 根据小说ID和章节索引获取所有角色对话
pub fn get_dialogues_by_chapter(
    app_handle: &tauri::AppHandle,
    novel_id: i64,
    chapter_index: i32,
) -> Result<Vec<DialogueData>> {
    let conn = db_start::get_database_connection(app_handle)?;

    let mut stmt = conn.prepare(
        "SELECT id, novel_id, chapter_index, dialogue_index, character_name, content, audio_path
         FROM dialogues WHERE novel_id = ?1 AND chapter_index = ?2 ORDER BY dialogue_index",
    )?;

    let rows = stmt.query_map(params![novel_id, chapter_index], |row| {
        Ok(DialogueData {
            id: Some(row.get(0)?),
            novel_id: row.get(1)?,
            chapter_index: row.get(2)?,
            dialogue_index: row.get(3)?,
            character_name: row.get(4)?,
            content: row.get(5)?,
            audio_path: row.get(6)?,
        })
    })?;

    let mut dialogues = Vec::new();
    for row in rows {
        dialogues.push(row?);
    }

    Ok(dialogues)
}

/// 根据小说ID获取所有角色对话
pub fn get_dialogues_by_novel_id(
    app_handle: &tauri::AppHandle,
    novel_id: i64,
) -> Result<Vec<DialogueData>> {
    let conn = db_start::get_database_connection(app_handle)?;

    let mut stmt = conn.prepare(
        "SELECT id, novel_id, chapter_index, dialogue_index, character_name, content, audio_path
         FROM dialogues WHERE novel_id = ?1 ORDER BY chapter_index, dialogue_index",
    )?;

    let rows = stmt.query_map(params![novel_id], |row| {
        Ok(DialogueData {
            id: Some(row.get(0)?),
            novel_id: row.get(1)?,
            chapter_index: row.get(2)?,
            dialogue_index: row.get(3)?,
            character_name: row.get(4)?,
            content: row.get(5)?,
            audio_path: row.get(6)?,
        })
    })?;

    let mut dialogues = Vec::new();
    for row in rows {
        dialogues.push(row?);
    }

    Ok(dialogues)
}

/// 删除角色对话
pub fn delete_dialogue(app_handle: &tauri::AppHandle, dialogue_id: i64) -> Result<()> {
    let conn = db_start::get_database_connection(app_handle)?;

    conn.execute("DELETE FROM dialogues WHERE id = ?1", params![dialogue_id])?;

    Ok(())
}

/// 根据小说ID删除所有角色对话
pub fn delete_dialogues_by_novel_id(app_handle: &tauri::AppHandle, novel_id: i64) -> Result<()> {
    let conn = db_start::get_database_connection(app_handle)?;

    conn.execute(
        "DELETE FROM dialogues WHERE novel_id = ?1",
        params![novel_id],
    )?;

    Ok(())
}

/// 更新角色对话音频路径
pub fn update_dialogue_audio_path(
    app_handle: &tauri::AppHandle,
    dialogue_id: i64,
    audio_path: Option<&str>,
) -> Result<()> {
    let conn = db_start::get_database_connection(app_handle)?;

    conn.execute(
        "UPDATE dialogues SET audio_path = ?1 WHERE id = ?2",
        params![audio_path, dialogue_id],
    )?;

    Ok(())
}

// ============================================================
// API Key 相关
// ============================================================

/// 大模型密钥数据
#[derive(Debug)]
pub struct ApiKeyData {
    pub id: Option<i64>,
    pub provider: String,
    pub api_key: String,
    pub base_url: Option<String>,
    pub model: Option<String>,
    pub is_active: bool,
}

/// 插入大模型密钥
pub fn insert_api_key(app_handle: &tauri::AppHandle, key_data: &ApiKeyData) -> Result<i64> {
    let conn = db_start::get_database_connection(app_handle)?;

    // 如果设置为活跃，先取消其他所有密钥的活跃状态
    if key_data.is_active {
        conn.execute("UPDATE api_keys SET is_active = 0", [])?;
    }

    conn.execute(
        "INSERT INTO api_keys (provider, api_key, base_url, model, is_active, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
        params![
            key_data.provider,
            key_data.api_key,
            key_data.base_url,
            key_data.model,
            key_data.is_active as i32,
        ],
    )?;

    Ok(conn.last_insert_rowid())
}

/// 获取所有大模型密钥
pub fn get_all_api_keys(app_handle: &tauri::AppHandle) -> Result<Vec<ApiKeyData>> {
    let conn = db_start::get_database_connection(app_handle)?;

    let mut stmt = conn.prepare(
        "SELECT id, provider, api_key, base_url, model, is_active
         FROM api_keys ORDER BY created_at DESC",
    )?;

    let rows = stmt.query_map([], |row| {
        Ok(ApiKeyData {
            id: Some(row.get(0)?),
            provider: row.get(1)?,
            api_key: row.get(2)?,
            base_url: row.get(3)?,
            model: row.get(4)?,
            is_active: row.get::<_, i32>(5)? != 0,
        })
    })?;

    let mut keys = Vec::new();
    for row in rows {
        keys.push(row?);
    }
    Ok(keys)
}

/// 获取当前活跃的大模型密钥
pub fn get_active_api_key(app_handle: &tauri::AppHandle) -> Result<Option<ApiKeyData>> {
    let conn = db_start::get_database_connection(app_handle)?;

    let mut stmt = conn.prepare(
        "SELECT id, provider, api_key, base_url, model, is_active
         FROM api_keys WHERE is_active = 1 LIMIT 1",
    )?;

    let mut rows = stmt.query_map([], |row| {
        Ok(ApiKeyData {
            id: Some(row.get(0)?),
            provider: row.get(1)?,
            api_key: row.get(2)?,
            base_url: row.get(3)?,
            model: row.get(4)?,
            is_active: row.get::<_, i32>(5)? != 0,
        })
    })?;

    if let Some(row) = rows.next() {
        Ok(Some(row?))
    } else {
        Ok(None)
    }
}

/// 更新大模型密钥
pub fn update_api_key(
    app_handle: &tauri::AppHandle,
    key_id: i64,
    key_data: &ApiKeyData,
) -> Result<()> {
    let conn = db_start::get_database_connection(app_handle)?;

    // 如果设置为活跃，先取消其他所有密钥的活跃状态
    if key_data.is_active {
        conn.execute("UPDATE api_keys SET is_active = 0", [])?;
    }

    conn.execute(
        "UPDATE api_keys SET provider = ?1, api_key = ?2, base_url = ?3,
         model = ?4, is_active = ?5, updated_at = CURRENT_TIMESTAMP WHERE id = ?6",
        params![
            key_data.provider,
            key_data.api_key,
            key_data.base_url,
            key_data.model,
            key_data.is_active as i32,
            key_id,
        ],
    )?;

    Ok(())
}

/// 删除大模型密钥
pub fn delete_api_key(app_handle: &tauri::AppHandle, key_id: i64) -> Result<()> {
    let conn = db_start::get_database_connection(app_handle)?;
    conn.execute("DELETE FROM api_keys WHERE id = ?1", params![key_id])?;
    Ok(())
}

/// 获取指定章节ID的完整内容
pub fn get_chapters_content_by_ids(
    app_handle: &tauri::AppHandle,
    chapter_ids: &[i32],
) -> Result<Vec<(i32, String, String)>> {
    let conn = db_start::get_database_connection(app_handle)?;

    if chapter_ids.is_empty() {
        return Ok(Vec::new());
    }

    // 构建带有占位符的 SQL
    let placeholders: Vec<String> = chapter_ids.iter().map(|_| "?".to_string()).collect();
    let sql = format!(
        "SELECT chapter_index, title, content FROM chapters WHERE chapter_index IN ({}) ORDER BY chapter_index",
        placeholders.join(",")
    );

    let mut stmt = conn.prepare(&sql)?;

    let params_refs: Vec<&dyn rusqlite::types::ToSql> = chapter_ids
        .iter()
        .map(|id| id as &dyn rusqlite::types::ToSql)
        .collect();

    let rows = stmt.query_map(params_refs.as_slice(), |row| {
        Ok((
            row.get::<_, i32>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
        ))
    })?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }

    Ok(results)
}

/// 更新角色对话角色名或内容
pub fn update_dialogue_content(
    app_handle: &tauri::AppHandle,
    dialogue_id: i64,
    character_name: Option<&str>,
    content: Option<&str>,
) -> Result<()> {
    let conn = db_start::get_database_connection(app_handle)?;

    let mut sql = String::from("UPDATE dialogues SET ");
    let mut set_clauses: Vec<String> = Vec::new();
    let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    if let Some(name) = character_name {
        set_clauses.push("character_name = ?".to_string());
        param_values.push(Box::new(name.to_string()));
    }
    if let Some(text) = content {
        set_clauses.push("content = ?".to_string());
        param_values.push(Box::new(text.to_string()));
    }

    if set_clauses.is_empty() {
        return Ok(());
    }

    sql.push_str(&set_clauses.join(", "));
    sql.push_str(" WHERE id = ?");
    param_values.push(Box::new(dialogue_id));

    conn.execute(
        &sql,
        rusqlite::params_from_iter(param_values.iter().map(|p| p.as_ref())),
    )?;

    Ok(())
}

// ============================================================
// TTS 剧本存储
// ============================================================

/// LLM 分片请求/响应数据（每次 API 调用一条）
#[derive(Debug)]
pub struct TtsChunkData {
    pub id: Option<i64>,
    pub novel_id: i64,
    pub chapter_id: i64,
    pub chunk_index: i32,
    pub system_prompt: Option<String>,
    pub user_content: Option<String>,
    pub raw_request: Option<String>,
    pub raw_response: Option<String>,
}

/// 插入 TTS 分片记录，返回新记录的 id
pub fn insert_tts_chunk(app_handle: &tauri::AppHandle, chunk: &TtsChunkData) -> Result<i64> {
    let conn = db_start::get_database_connection(app_handle)?;

    conn.execute(
        "INSERT INTO tts_chunks (novel_id, chapter_id, chunk_index, system_prompt, user_content, raw_request, raw_response, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, CURRENT_TIMESTAMP)",
        params![
            chunk.novel_id,
            chunk.chapter_id,
            chunk.chunk_index,
            chunk.system_prompt,
            chunk.user_content,
            chunk.raw_request,
            chunk.raw_response,
        ],
    )?;

    Ok(conn.last_insert_rowid())
}

/// TTS 剧本场景数据（结构化，每场景一条）
#[derive(Debug)]
pub struct TtsSceneData {
    pub chunk_id: Option<i64>,
    pub novel_id: i64,
    pub chapter_id: i64,
    pub chunk_index: i32,
    pub scene_index: i32,
    pub scene_type: String,
    pub content: String,
    pub character_name: Option<String>,
    pub emotion: String,
    pub speed: f64,
    pub pitch: f64,
    pub pause_duration: f64,
}

/// 批量插入 TTS 剧本场景
pub fn insert_tts_scenes_batch(
    app_handle: &tauri::AppHandle,
    scenes: &[TtsSceneData],
) -> Result<()> {
    let mut conn = db_start::get_database_connection(app_handle)?;

    let tx = conn.transaction()?;

    for scene in scenes {
        tx.execute(
            "INSERT INTO tts_scripts (chunk_id, novel_id, chapter_id, chunk_index, scene_index, scene_type, content, character_name, emotion, speed, pitch, pause_duration, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, CURRENT_TIMESTAMP)",
            params![
                scene.chunk_id,
                scene.novel_id,
                scene.chapter_id,
                scene.chunk_index,
                scene.scene_index,
                scene.scene_type,
                scene.content,
                scene.character_name,
                scene.emotion,
                scene.speed,
                scene.pitch,
                scene.pause_duration,
            ],
        )?;
    }

    tx.commit()?;
    Ok(())
}

/// 查询 TTS 剧本（按小说ID）
pub fn get_tts_scripts_by_novel(
    app_handle: &tauri::AppHandle,
    novel_id: i64,
) -> Result<Vec<TtsSceneData>> {
    let conn = db_start::get_database_connection(app_handle)?;

    let mut stmt = conn.prepare(
        "SELECT chunk_id, novel_id, chapter_id, chunk_index, scene_index, scene_type, content, character_name, emotion, speed, pitch, pause_duration
         FROM tts_scripts WHERE novel_id = ?1 ORDER BY chapter_id, chunk_index, scene_index"
    )?;

    let rows = stmt.query_map(params![novel_id], |row| {
        Ok(TtsSceneData {
            chunk_id: row.get(0)?,
            novel_id: row.get(1)?,
            chapter_id: row.get(2)?,
            chunk_index: row.get(3)?,
            scene_index: row.get(4)?,
            scene_type: row.get(5)?,
            content: row.get(6)?,
            character_name: row.get(7)?,
            emotion: row.get(8)?,
            speed: row.get(9)?,
            pitch: row.get(10)?,
            pause_duration: row.get(11)?,
        })
    })?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }
    Ok(results)
}

/// 查询指定章节的 TTS 剧本
pub fn get_tts_scripts_by_chapter(
    app_handle: &tauri::AppHandle,
    chapter_id: i64,
) -> Result<Vec<TtsSceneData>> {
    let conn = db_start::get_database_connection(app_handle)?;

    let mut stmt = conn.prepare(
        "SELECT chunk_id, novel_id, chapter_id, chunk_index, scene_index, scene_type, content, character_name, emotion, speed, pitch, pause_duration
         FROM tts_scripts WHERE chapter_id = ?1 ORDER BY chunk_index, scene_index"
    )?;

    let rows = stmt.query_map(params![chapter_id], |row| {
        Ok(TtsSceneData {
            chunk_id: row.get(0)?,
            novel_id: row.get(1)?,
            chapter_id: row.get(2)?,
            chunk_index: row.get(3)?,
            scene_index: row.get(4)?,
            scene_type: row.get(5)?,
            content: row.get(6)?,
            character_name: row.get(7)?,
            emotion: row.get(8)?,
            speed: row.get(9)?,
            pitch: row.get(10)?,
            pause_duration: row.get(11)?,
        })
    })?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }
    Ok(results)
}

/// TTS 章节摘要
#[derive(Debug, Clone, serde::Serialize, tauri_ts_generator::TS)]
#[ts(export)]
pub struct ChapterTtsSummary {
    pub chapter_id: i64,
    pub characters: Vec<String>,
    pub scene_count: i64,
    pub dialogue_count: i64,
    pub total_text_length: i64,
}

/// 查询指定章节的 TTS 摘要（角色列表、场景数等）
pub fn get_chapter_tts_summary(
    app_handle: &tauri::AppHandle,
    chapter_id: i64,
) -> Result<ChapterTtsSummary> {
    let conn = db_start::get_database_connection(app_handle)?;

    // 统计
    let (scene_count, dialogue_count, total_text_length): (i64, i64, i64) = conn.query_row(
        "SELECT COUNT(*),
                SUM(CASE WHEN scene_type = 'dialogue' THEN 1 ELSE 0 END),
                COALESCE(SUM(LENGTH(content)), 0)
         FROM tts_scripts WHERE chapter_id = ?1",
        params![chapter_id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    )?;

    // 去重角色
    let mut stmt = conn.prepare(
        "SELECT DISTINCT character_name FROM tts_scripts
         WHERE chapter_id = ?1 AND character_name IS NOT NULL AND character_name != ''
         ORDER BY character_name",
    )?;
    let characters: Vec<String> = stmt
        .query_map(params![chapter_id], |row| row.get::<_, String>(0))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(ChapterTtsSummary {
        chapter_id,
        characters,
        scene_count,
        dialogue_count,
        total_text_length,
    })
}

// ============================================================
// 章节查询（含完整信息）
// ============================================================

/// 获取指定章节的完整信息（含 novel_id 和 chapter_id）
/// 返回 (chapter_index, title, content, novel_id, chapter_id)
pub fn get_chapters_full_info_by_ids(
    app_handle: &tauri::AppHandle,
    chapter_ids: &[i32],
) -> Result<Vec<ChapterFullInfo>> {
    let conn = db_start::get_database_connection(app_handle)?;

    if chapter_ids.is_empty() {
        return Ok(Vec::new());
    }

    let placeholders: Vec<String> = chapter_ids.iter().map(|_| "?".to_string()).collect();
    let sql = format!(
        "SELECT chapter_index, title, content, novel_id, id FROM chapters WHERE id IN ({}) ORDER BY chapter_index",
        placeholders.join(",")
    );

    let mut stmt = conn.prepare(&sql)?;

    let params_refs: Vec<&dyn rusqlite::types::ToSql> = chapter_ids
        .iter()
        .map(|id| id as &dyn rusqlite::types::ToSql)
        .collect();

    let rows = stmt.query_map(params_refs.as_slice(), |row| {
        Ok((
            row.get::<_, i32>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, i64>(3)?,
            row.get::<_, i64>(4)?,
        ))
    })?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }

    Ok(results)
}

// ============================================================
// Character Voice 相关
// ============================================================

/// 角色语音配置
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, tauri_ts_generator::TS)]
#[ts(export)]
pub struct CharacterVoiceData {
    pub id: Option<i64>,
    pub character_name: String,
    pub ref_audio_path: String,
    pub prompt_text: Option<String>,
    pub prompt_lang: String,
    pub text_lang: String,
    pub gpt_model: Option<String>,
    pub sovits_model: Option<String>,
    pub api_base_url: String,
    pub config_path: Option<String>,
}

/// 插入角色语音配置
pub fn insert_character_voice(
    app_handle: &tauri::AppHandle,
    data: &CharacterVoiceData,
) -> Result<i64> {
    let conn = db_start::get_database_connection(app_handle)?;

    conn.execute(
        "INSERT INTO character_voices (character_name, ref_audio_path, prompt_text, prompt_lang, text_lang, gpt_model, sovits_model, api_base_url, config_path, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
        params![
            data.character_name,
            data.ref_audio_path,
            data.prompt_text,
            data.prompt_lang,
            data.text_lang,
            data.gpt_model,
            data.sovits_model,
            data.api_base_url,
            data.config_path,
        ],
    )?;

    Ok(conn.last_insert_rowid())
}

/// 获取所有角色语音配置
pub fn get_all_character_voices(app_handle: &tauri::AppHandle) -> Result<Vec<CharacterVoiceData>> {
    let conn = db_start::get_database_connection(app_handle)?;

    let mut stmt = conn.prepare(
        "SELECT id, character_name, ref_audio_path, prompt_text, prompt_lang, text_lang, gpt_model, sovits_model, api_base_url, config_path
         FROM character_voices ORDER BY character_name"
    )?;

    let rows = stmt.query_map([], |row| {
        Ok(CharacterVoiceData {
            id: Some(row.get(0)?),
            character_name: row.get(1)?,
            ref_audio_path: row.get(2)?,
            prompt_text: row.get(3)?,
            prompt_lang: row.get(4)?,
            text_lang: row.get(5)?,
            gpt_model: row.get(6)?,
            sovits_model: row.get(7)?,
            api_base_url: row.get(8)?,
            config_path: row.get(9)?,
        })
    })?;

    let mut list = Vec::new();
    for row in rows {
        list.push(row?);
    }
    Ok(list)
}

/// 更新角色语音配置
pub fn update_character_voice(
    app_handle: &tauri::AppHandle,
    id: i64,
    data: &CharacterVoiceData,
) -> Result<()> {
    let conn = db_start::get_database_connection(app_handle)?;

    conn.execute(
        "UPDATE character_voices SET character_name=?1, ref_audio_path=?2, prompt_text=?3,
         prompt_lang=?4, text_lang=?5, gpt_model=?6, sovits_model=?7, api_base_url=?8,
         config_path=?9, updated_at=CURRENT_TIMESTAMP WHERE id=?10",
        params![
            data.character_name,
            data.ref_audio_path,
            data.prompt_text,
            data.prompt_lang,
            data.text_lang,
            data.gpt_model,
            data.sovits_model,
            data.api_base_url,
            data.config_path,
            id,
        ],
    )?;
    Ok(())
}

/// 根据 id 获取角色语音配置
pub fn get_character_voice_by_id(
    app_handle: &tauri::AppHandle,
    id: i64,
) -> Result<CharacterVoiceData> {
    let conn = db_start::get_database_connection(app_handle)?;
    conn.query_row(
        "SELECT id, character_name, ref_audio_path, prompt_text, prompt_lang, text_lang,
                gpt_model, sovits_model, api_base_url, config_path
         FROM character_voices WHERE id = ?1",
        params![id],
        |row| {
            Ok(CharacterVoiceData {
                id: Some(row.get(0)?),
                character_name: row.get(1)?,
                ref_audio_path: row.get(2)?,
                prompt_text: row.get(3)?,
                prompt_lang: row.get(4)?,
                text_lang: row.get(5)?,
                gpt_model: row.get(6)?,
                sovits_model: row.get(7)?,
                api_base_url: row.get(8)?,
                config_path: row.get(9)?,
            })
        },
    )
}

/// 删除角色语音配置
pub fn delete_character_voice(app_handle: &tauri::AppHandle, id: i64) -> Result<()> {
    let conn = db_start::get_database_connection(app_handle)?;
    conn.execute("DELETE FROM character_voices WHERE id = ?1", params![id])?;
    Ok(())
}

/// 已生成 TTS 剧本的章节列表（含小说信息）
#[derive(Debug, serde::Serialize, tauri_ts_generator::TS)]
#[ts(export)]
pub struct TtsGeneratedChapter {
    pub chapter_id: i64,
    pub novel_id: i64,
    pub novel_title: String,
    pub novel_author: Option<String>,
    pub chapter_index: i32,
    pub chapter_title: String,
    pub scene_count: i64,
}

/// 查询所有已生成 TTS 剧本的章节
pub fn get_tts_generated_chapters(
    app_handle: &tauri::AppHandle,
) -> Result<Vec<TtsGeneratedChapter>> {
    let conn = db_start::get_database_connection(app_handle)?;

    let mut stmt = conn.prepare(
        "SELECT c.id, c.novel_id, c.chapter_index, c.title,
                n.title, n.author,
                (SELECT COUNT(*) FROM tts_scripts WHERE chapter_id = c.id) AS scene_count
         FROM chapters c
         JOIN novels n ON c.novel_id = n.id
         WHERE c.tts_generated = 1
         ORDER BY n.title, c.chapter_index",
    )?;

    let rows = stmt.query_map([], |row| {
        Ok(TtsGeneratedChapter {
            chapter_id: row.get(0)?,
            novel_id: row.get(1)?,
            chapter_index: row.get(2)?,
            chapter_title: row.get(3)?,
            novel_title: row.get(4)?,
            novel_author: row.get(5)?,
            scene_count: row.get(6)?,
        })
    })?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }
    Ok(results)
}

// ============================================================
// Chapter Character Voice Mapping
// ============================================================

/// 章节角色 ↔ 语音配置映射
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChapterCharacterVoiceMap {
    pub id: Option<i64>,
    pub chapter_id: i64,
    pub character_name: String,
    pub voice_id: i64,
}

/// 所有章节（含小说信息），用于 TtsGenerate 页面展示
#[derive(Debug, Clone, serde::Serialize)]
pub struct ChapterWithNovel {
    pub chapter_id: i64,
    pub novel_id: i64,
    pub novel_title: String,
    pub novel_author: Option<String>,
    pub chapter_index: i32,
    pub chapter_title: String,
    pub tts_generated: bool,
    pub scene_count: i64,
}

/// 查询所有章节（含小说信息），按小说分组、章节排序
pub fn get_all_chapters_grouped_by_novel(
    app_handle: &tauri::AppHandle,
) -> Result<Vec<ChapterWithNovel>> {
    let conn = db_start::get_database_connection(app_handle)?;

    let mut stmt = conn.prepare(
        "SELECT c.id, c.novel_id, c.chapter_index, c.title, c.tts_generated,
                n.title, n.author,
                (SELECT COUNT(*) FROM tts_scripts WHERE chapter_id = c.id) AS scene_count
         FROM chapters c
         JOIN novels n ON c.novel_id = n.id
         ORDER BY n.title, c.chapter_index",
    )?;

    let rows = stmt.query_map([], |row| {
        Ok(ChapterWithNovel {
            chapter_id: row.get(0)?,
            novel_id: row.get(1)?,
            chapter_index: row.get(2)?,
            chapter_title: row.get(3)?,
            tts_generated: row.get::<_, i32>(4)? != 0,
            novel_title: row.get(5)?,
            novel_author: row.get(6)?,
            scene_count: row.get(7)?,
        })
    })?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }
    Ok(results)
}

/// 获取指定章节的角色→语音映射列表
pub fn get_character_voice_mappings_for_chapter(
    app_handle: &tauri::AppHandle,
    chapter_id: i64,
) -> Result<Vec<ChapterCharacterVoiceMap>> {
    let conn = db_start::get_database_connection(app_handle)?;

    let mut stmt = conn.prepare(
        "SELECT id, chapter_id, character_name, voice_id
         FROM chapter_character_voice_map WHERE chapter_id = ?1
         ORDER BY character_name",
    )?;

    let rows = stmt.query_map(params![chapter_id], |row| {
        Ok(ChapterCharacterVoiceMap {
            id: Some(row.get(0)?),
            chapter_id: row.get(1)?,
            character_name: row.get(2)?,
            voice_id: row.get(3)?,
        })
    })?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }
    Ok(results)
}

/// 保存章节的角色→语音映射（先删旧映射再批量插入）
pub fn save_character_voice_mappings_for_chapter(
    app_handle: &tauri::AppHandle,
    chapter_id: i64,
    mappings: &[(String, i64)],
) -> Result<()> {
    let mut conn = db_start::get_database_connection(app_handle)?;

    let tx = conn.transaction()?;

    // 删除旧的映射
    tx.execute(
        "DELETE FROM chapter_character_voice_map WHERE chapter_id = ?1",
        params![chapter_id],
    )?;

    // 插入新映射
    for (character_name, voice_id) in mappings {
        tx.execute(
            "INSERT INTO chapter_character_voice_map (chapter_id, character_name, voice_id, created_at)
             VALUES (?1, ?2, ?3, CURRENT_TIMESTAMP)",
            params![chapter_id, character_name, voice_id],
        )?;
    }

    tx.commit()?;
    Ok(())
}

/// TTS 场景（含 id），用于音频生成
#[derive(Debug, Clone)]
pub struct TtsSceneWithId {
    pub id: i64,
    pub novel_id: i64,
    pub chapter_id: i64,
    pub scene_index: i32,
    pub character_name: Option<String>,
    pub content: String,
    pub emotion: String,
    pub speed: f64,
    pub pitch: f64,
}

/// 获取指定章节的所有 TTS 场景（含 id）
pub fn get_tts_scenes_with_id_by_chapter(
    app_handle: &tauri::AppHandle,
    chapter_id: i64,
) -> Result<Vec<TtsSceneWithId>> {
    let conn = db_start::get_database_connection(app_handle)?;

    let mut stmt = conn.prepare(
        "SELECT id, novel_id, chapter_id, scene_index, character_name, content, emotion, speed, pitch
         FROM tts_scripts WHERE chapter_id = ?1
         ORDER BY chunk_index, scene_index"
    )?;

    let rows = stmt.query_map(params![chapter_id], |row| {
        Ok(TtsSceneWithId {
            id: row.get(0)?,
            novel_id: row.get(1)?,
            chapter_id: row.get(2)?,
            scene_index: row.get(3)?,
            character_name: row.get(4)?,
            content: row.get(5)?,
            emotion: row.get(6)?,
            speed: row.get(7)?,
            pitch: row.get(8)?,
        })
    })?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }
    Ok(results)
}

/// 更新单条 TTS 场景的音频路径
pub fn update_tts_script_audio_path(
    app_handle: &tauri::AppHandle,
    script_id: i64,
    audio_path: Option<&str>,
) -> Result<()> {
    let conn = db_start::get_database_connection(app_handle)?;

    conn.execute(
        "UPDATE tts_scripts SET audio_path = ?1 WHERE id = ?2",
        params![audio_path, script_id],
    )?;

    Ok(())
}

/// 更新章节的 audio_path 字段（标记整章音频已生成）
pub fn update_chapter_audio_path(
    app_handle: &tauri::AppHandle,
    chapter_id: i64,
    audio_path: Option<&str>,
) -> Result<()> {
    let conn = db_start::get_database_connection(app_handle)?;

    conn.execute(
        "UPDATE chapters SET audio_path = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = ?2",
        params![audio_path, chapter_id],
    )?;

    Ok(())
}

/// 通过 novel_id + chapter_index 定位章节（比用 id 更可靠）
/// 返回 (chapter_index, title, content, novel_id, chapter_id)
pub fn get_chapters_by_novel_and_indices(
    app_handle: &tauri::AppHandle,
    novel_id: i64,
    chapter_indices: &[i32],
) -> Result<Vec<ChapterFullInfo>> {
    let conn = db_start::get_database_connection(app_handle)?;

    if chapter_indices.is_empty() {
        return Ok(Vec::new());
    }

    let placeholders: Vec<String> = chapter_indices.iter().map(|_| "?".to_string()).collect();
    let sql = format!(
        "SELECT chapter_index, title, content, novel_id, id FROM chapters \
         WHERE novel_id = ?1 AND chapter_index IN ({}) ORDER BY chapter_index",
        placeholders.join(",")
    );

    let mut stmt = conn.prepare(&sql)?;

    // 第一个参数 novel_id，后面跟所有 chapter_index
    let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
    param_values.push(Box::new(novel_id));
    for idx in chapter_indices {
        param_values.push(Box::new(*idx));
    }

    let params_refs: Vec<&dyn rusqlite::types::ToSql> =
        param_values.iter().map(|p| p.as_ref()).collect();
    let rows = stmt.query_map(params_refs.as_slice(), |row| {
        Ok((
            row.get::<_, i32>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, i64>(3)?,
            row.get::<_, i64>(4)?,
        ))
    })?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }

    Ok(results)
}

// ============================================================
// 音频播放相关数据结构与查询
// ============================================================

/// 有已生成音频的小说信息
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, tauri_ts_generator::TS)]
#[ts(export)]
pub struct NovelWithAudioInfo {
    pub novel_id: i64,
    pub novel_title: String,
    pub novel_author: Option<String>,
    pub cover_image_path: Option<String>,
    pub chapter_count: i64,
}

/// 有已生成音频的章节信息
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, tauri_ts_generator::TS)]
#[ts(export)]
pub struct ChapterWithAudioInfo {
    pub chapter_id: i64,
    pub novel_id: i64,
    pub chapter_index: i32,
    pub chapter_title: String,
    pub scene_count: i64,
}

/// 有音频文件的 TTS 场景（用于播放器歌词展示）
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, tauri_ts_generator::TS)]
#[ts(export)]
pub struct TtsSceneWithAudio {
    pub id: i64,
    pub novel_id: i64,
    pub chapter_id: i64,
    pub scene_index: i32,
    pub scene_type: String,
    pub content: String,
    pub character_name: Option<String>,
    pub emotion: String,
    pub speed: f64,
    pub pause_duration: f64,
    pub audio_path: Option<String>,
}

/// 查询所有有已生成音频的小说
pub fn get_novels_with_audio(app_handle: &tauri::AppHandle) -> Result<Vec<NovelWithAudioInfo>> {
    let conn = db_start::get_database_connection(app_handle)?;
    let mut stmt = conn.prepare(
        "SELECT n.id, n.title, n.author, n.cover_image_path,
                (SELECT COUNT(*) FROM chapters c2
                 WHERE c2.novel_id = n.id AND c2.tts_generated = 1
                   AND c2.audio_path IS NOT NULL AND c2.audio_path != '') AS chapter_count
         FROM novels n
         JOIN chapters c ON c.novel_id = n.id
         WHERE c.tts_generated = 1 AND c.audio_path IS NOT NULL AND c.audio_path != ''
         GROUP BY n.id
         ORDER BY n.title",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(NovelWithAudioInfo {
            novel_id: row.get(0)?,
            novel_title: row.get(1)?,
            novel_author: row.get(2)?,
            cover_image_path: row.get(3)?,
            chapter_count: row.get(4)?,
        })
    })?;
    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }
    Ok(results)
}

/// 查询指定小说下有已生成音频的章节
pub fn get_chapters_with_audio(
    app_handle: &tauri::AppHandle,
    novel_id: i64,
) -> Result<Vec<ChapterWithAudioInfo>> {
    let conn = db_start::get_database_connection(app_handle)?;
    let mut stmt = conn.prepare(
        "SELECT c.id, c.novel_id, c.chapter_index, c.title,
                (SELECT COUNT(*) FROM tts_scripts
                 WHERE chapter_id = c.id AND audio_path IS NOT NULL AND audio_path != '') AS scene_count
         FROM chapters c
         WHERE c.novel_id = ?1 AND c.tts_generated = 1
           AND c.audio_path IS NOT NULL AND c.audio_path != ''
         ORDER BY c.chapter_index"
    )?;
    let rows = stmt.query_map(params![novel_id], |row| {
        Ok(ChapterWithAudioInfo {
            chapter_id: row.get(0)?,
            novel_id: row.get(1)?,
            chapter_index: row.get(2)?,
            chapter_title: row.get(3)?,
            scene_count: row.get(4)?,
        })
    })?;
    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }
    Ok(results)
}

/// 查询指定章节所有有音频的场景
pub fn get_tts_scenes_with_audio_by_chapter(
    app_handle: &tauri::AppHandle,
    chapter_id: i64,
) -> Result<Vec<TtsSceneWithAudio>> {
    let conn = db_start::get_database_connection(app_handle)?;
    let mut stmt = conn.prepare(
        "SELECT id, novel_id, chapter_id, scene_index, scene_type, content,
                character_name, emotion, speed, pause_duration, audio_path
         FROM tts_scripts WHERE chapter_id = ?1
           AND audio_path IS NOT NULL AND audio_path != ''
         ORDER BY chunk_index, scene_index",
    )?;
    let rows = stmt.query_map(params![chapter_id], |row| {
        Ok(TtsSceneWithAudio {
            id: row.get(0)?,
            novel_id: row.get(1)?,
            chapter_id: row.get(2)?,
            scene_index: row.get(3)?,
            scene_type: row.get(4)?,
            content: row.get(5)?,
            character_name: row.get(6)?,
            emotion: row.get(7)?,
            speed: row.get(8)?,
            pause_duration: row.get(9)?,
            audio_path: row.get(10)?,
        })
    })?;
    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }
    Ok(results)
}

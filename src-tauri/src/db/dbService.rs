use std::path::PathBuf;
use rusqlite::{Result, params};

use super::dbStart;

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
}

/// 插入小说到数据库
/// 返回新插入小说的ID
pub fn insert_novel(app_handle: &tauri::AppHandle, novel_data: &NovelData) -> Result<i64> {
    let mut conn = dbStart::get_database_connection(app_handle)?;

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
    let mut conn = dbStart::get_database_connection(app_handle)?;

    conn.execute(
        "INSERT INTO chapters (novel_id, chapter_index, title, content, audio_path, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
        params![
            chapter_data.novel_id,
            chapter_data.chapter_index,
            chapter_data.title,
            chapter_data.content,
            chapter_data.audio_path,
        ],
    )?;

    Ok(())
}

/// 批量插入章节
pub fn insert_chapters_batch(app_handle: &tauri::AppHandle, chapters: &[ChapterData]) -> Result<()> {
    let mut conn = dbStart::get_database_connection(app_handle)?;

    // 开始事务
    let tx = conn.transaction()?;

    for chapter in chapters {
        tx.execute(
            "INSERT INTO chapters (novel_id, chapter_index, title, content, audio_path, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
            params![
                chapter.novel_id,
                chapter.chapter_index,
                chapter.title,
                chapter.content,
                chapter.audio_path,
            ],
        )?;
    }

    tx.commit()?;
    Ok(())
}

/// 根据文件路径查找小说ID
pub fn find_novel_by_file_path(app_handle: &tauri::AppHandle, file_path: &PathBuf) -> Result<Option<i64>> {
    let mut conn = dbStart::get_database_connection(app_handle)?;

    let mut stmt = conn.prepare("SELECT id FROM novels WHERE file_path = ?1")?;
    let mut rows = stmt.query(params![file_path.to_string_lossy()])?;

    if let Some(row) = rows.next()? {
        Ok(Some(row.get(0)?))
    } else {
        Ok(None)
    }
}

/// 获取小说所有章节
pub fn get_chapters_by_novel_id(app_handle: &tauri::AppHandle, novel_id: i64) -> Result<Vec<ChapterData>> {
    let mut conn = dbStart::get_database_connection(app_handle)?;

    let mut stmt = conn.prepare(
        "SELECT novel_id, chapter_index, title, content, audio_path
         FROM chapters WHERE novel_id = ?1 ORDER BY chapter_index"
    )?;

    let rows = stmt.query_map(params![novel_id], |row| {
        Ok(ChapterData {
            novel_id: row.get(0)?,
            chapter_index: row.get(1)?,
            title: row.get(2)?,
            content: row.get(3)?,
            audio_path: row.get(4)?,
        })
    })?;

    let mut chapters = Vec::new();
    for row in rows {
        chapters.push(row?);
    }

    Ok(chapters)
}

/// 删除小说及其所有章节
pub fn delete_novel(app_handle: &tauri::AppHandle, novel_id: i64) -> Result<()> {
    let mut conn = dbStart::get_database_connection(app_handle)?;

    // 由于外键设置了 ON DELETE CASCADE，删除小说会自动删除相关章节
    conn.execute("DELETE FROM novels WHERE id = ?1", params![novel_id])?;

    Ok(())
}

/// 更新小说信息
pub fn update_novel(app_handle: &tauri::AppHandle, novel_id: i64, title: &str, author: Option<&str>) -> Result<()> {
    let mut conn = dbStart::get_database_connection(app_handle)?;

    conn.execute(
        "UPDATE novels SET title = ?1, author = ?2, updated_at = CURRENT_TIMESTAMP WHERE id = ?3",
        params![title, author, novel_id],
    )?;

    Ok(())
}

/// 获取所有小说
pub fn get_all_novels(app_handle: &tauri::AppHandle) -> Result<Vec<NovelData>> {
    let mut conn = dbStart::get_database_connection(app_handle)?;

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
    let mut conn = dbStart::get_database_connection(app_handle)?;

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
pub fn insert_illustration(app_handle: &tauri::AppHandle, illustration_data: &IllustrationData) -> Result<i64> {
    let mut conn = dbStart::get_database_connection(app_handle)?;

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
pub fn insert_illustrations_batch(app_handle: &tauri::AppHandle, illustrations: &[IllustrationData]) -> Result<()> {
    let mut conn = dbStart::get_database_connection(app_handle)?;

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
pub fn get_illustrations_by_novel_id(app_handle: &tauri::AppHandle, novel_id: i64) -> Result<Vec<IllustrationData>> {
    let mut conn = dbStart::get_database_connection(app_handle)?;

    let mut stmt = conn.prepare(
        "SELECT id, novel_id, image_path, description, chapter_index
         FROM illustrations WHERE novel_id = ?1 ORDER BY created_at"
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
    let mut conn = dbStart::get_database_connection(app_handle)?;

    conn.execute("DELETE FROM illustrations WHERE id = ?1", params![illustration_id])?;

    Ok(())
}

/// 根据小说ID删除所有插图
pub fn delete_illustrations_by_novel_id(app_handle: &tauri::AppHandle, novel_id: i64) -> Result<()> {
    let mut conn = dbStart::get_database_connection(app_handle)?;

    conn.execute("DELETE FROM illustrations WHERE novel_id = ?1", params![novel_id])?;

    Ok(())
}

/// 更新插图描述
pub fn update_illustration_description(app_handle: &tauri::AppHandle, illustration_id: i64, description: Option<&str>) -> Result<()> {
    let mut conn = dbStart::get_database_connection(app_handle)?;

    conn.execute(
        "UPDATE illustrations SET description = ?1 WHERE id = ?2",
        params![description, illustration_id],
    )?;

    Ok(())
}

/// 更新小说封面图片路径
pub fn update_novel_cover_image_path(app_handle: &tauri::AppHandle, novel_id: i64, cover_image_path: Option<&str>) -> Result<()> {
    let mut conn = dbStart::get_database_connection(app_handle)?;

    conn.execute(
        "UPDATE novels SET cover_image_path = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = ?2",
        params![cover_image_path, novel_id],
    )?;

    Ok(())
}
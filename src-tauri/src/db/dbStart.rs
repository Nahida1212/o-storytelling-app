use std::path::PathBuf;
use rusqlite::{Connection, Result};
use tauri::Manager;

/// 获取数据库文件路径
pub fn get_database_path(app_handle: &tauri::AppHandle) -> PathBuf {
    // 获取应用本地数据目录
    let local_data_dir = app_handle
        .path()
        .local_data_dir()
        .expect("无法获取本地数据目录");

    // 创建项目特定文件夹
    let app_data_dir = local_data_dir.join("o-storytelling-app");

    // 确保目录存在
    std::fs::create_dir_all(&app_data_dir).expect("无法创建应用数据目录");

    // 数据库文件路径
    app_data_dir.join("storytelling.db")
}

/// 获取数据库连接
pub fn get_database_connection(app_handle: &tauri::AppHandle) -> Result<Connection> {
    let db_path = get_database_path(app_handle);
    Connection::open(&db_path)
}

pub fn initialize_database(app_handle: &tauri::AppHandle) -> Result<()> {
    let mut conn = get_database_connection(app_handle)?;

    // 创建 novels 表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS novels (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            author TEXT,
            file_path TEXT UNIQUE NOT NULL,
            cover_image BLOB,
            cover_image_path TEXT,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;


    // 创建 chapters 表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS chapters (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            novel_id INTEGER NOT NULL,
            chapter_index INTEGER NOT NULL,
            title TEXT NOT NULL,
            content TEXT,
            audio_path TEXT,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (novel_id) REFERENCES novels(id) ON DELETE CASCADE,
            UNIQUE(novel_id, chapter_index)
        )",
        [],
    )?;

    // 创建 illustrations 表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS illustrations (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            novel_id INTEGER NOT NULL,
            image_path TEXT NOT NULL,
            description TEXT,
            chapter_index INTEGER, -- 关联的章节索引（可为空）
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (novel_id) REFERENCES novels(id) ON DELETE CASCADE
        )",
        [],
    )?;

    // 输出日志
    let db_path = get_database_path(app_handle);
    println!("初始化数据库成功: {:?}", db_path);

    Ok(())
}
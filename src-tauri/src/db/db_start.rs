use rusqlite::{Connection, Result};
use std::path::PathBuf;
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
    let conn = get_database_connection(app_handle)?;

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
            tts_generated INTEGER DEFAULT 0,
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

    // 创建 dialogues 表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS dialogues (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            novel_id INTEGER NOT NULL,
            chapter_index INTEGER NOT NULL,
            dialogue_index INTEGER NOT NULL,
            character_name TEXT NOT NULL,
            content TEXT NOT NULL,
            audio_path TEXT,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (novel_id) REFERENCES novels(id) ON DELETE CASCADE,
            UNIQUE(novel_id, chapter_index, dialogue_index)
        )",
        [],
    )?;

    // 创建 api_keys 表（用于存储大模型密钥）
    conn.execute(
        "CREATE TABLE IF NOT EXISTS api_keys (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            provider TEXT NOT NULL,
            api_key TEXT NOT NULL,
            base_url TEXT,
            model TEXT,
            is_active INTEGER DEFAULT 0,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    // 创建 tts_chunks 表（存储每次 LLM API 调用的原始请求和响应，每分片一条）
    conn.execute(
        "CREATE TABLE IF NOT EXISTS tts_chunks (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            novel_id INTEGER NOT NULL,
            chapter_id INTEGER NOT NULL,
            chunk_index INTEGER NOT NULL,
            system_prompt TEXT,
            user_content TEXT,
            raw_request TEXT,
            raw_response TEXT,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (novel_id) REFERENCES novels(id) ON DELETE CASCADE
        )",
        [],
    )?;

    // 创建 tts_scripts 表（场景级别的结构化数据，每场景一条）
    conn.execute(
        "CREATE TABLE IF NOT EXISTS tts_scripts (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            chunk_id INTEGER,
            novel_id INTEGER NOT NULL,
            chapter_id INTEGER NOT NULL,
            chunk_index INTEGER NOT NULL,
            scene_index INTEGER NOT NULL,
            scene_type TEXT NOT NULL,
            content TEXT NOT NULL,
            character_name TEXT,
            emotion TEXT NOT NULL DEFAULT 'neutral',
            speed REAL NOT NULL DEFAULT 1.0,
            pitch REAL NOT NULL DEFAULT 1.0,
            pause_duration REAL NOT NULL DEFAULT 0.5,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (novel_id) REFERENCES novels(id) ON DELETE CASCADE
        )",
        [],
    )?;

    // 迁移：为旧 tts_scripts 表添加 chunk_id 列（已存在则跳过）
    match conn.execute(
        "ALTER TABLE tts_scripts ADD COLUMN chunk_id INTEGER REFERENCES tts_chunks(id)",
        [],
    ) {
        Ok(_) => println!("[dbStart] 迁移：已添加 chunk_id 列"),
        Err(e) => println!("[dbStart] 迁移：chunk_id 列已存在 ({})", e),
    }

    // 创建 character_voices 表（角色语音配置）
    conn.execute(
        "CREATE TABLE IF NOT EXISTS character_voices (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            character_name TEXT NOT NULL UNIQUE,
            ref_audio_path TEXT NOT NULL,
            prompt_text TEXT,
            prompt_lang TEXT NOT NULL DEFAULT 'zh',
            text_lang TEXT NOT NULL DEFAULT 'zh',
            gpt_model TEXT,
            sovits_model TEXT,
            api_base_url TEXT NOT NULL DEFAULT 'http://127.0.0.1:9880',
            config_path TEXT,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;
    println!("[dbStart] 已创建 character_voices 表");

    // 创建 chapter_character_voice_map 表（章节角色 ↔ 语音配置映射）
    conn.execute(
        "CREATE TABLE IF NOT EXISTS chapter_character_voice_map (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            chapter_id INTEGER NOT NULL,
            character_name TEXT NOT NULL,
            voice_id INTEGER NOT NULL,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (chapter_id) REFERENCES chapters(id) ON DELETE CASCADE,
            FOREIGN KEY (voice_id) REFERENCES character_voices(id) ON DELETE CASCADE,
            UNIQUE(chapter_id, character_name)
        )",
        [],
    )?;
    println!("[dbStart] 已创建 chapter_character_voice_map 表");

    // 输出日志
    let db_path = get_database_path(app_handle);
    println!("初始化数据库成功: {:?}", db_path);

    // 迁移：为旧 chapters 表添加 tts_generated 列
    match conn.execute(
        "ALTER TABLE chapters ADD COLUMN tts_generated INTEGER DEFAULT 0",
        [],
    ) {
        Ok(_) => println!("[dbStart] 迁移：已添加 tts_generated 列"),
        Err(e) => println!("[dbStart] 迁移：tts_generated 列已存在 ({})", e),
    }

    // 迁移：为旧 tts_scripts 表添加 audio_path 列
    match conn.execute("ALTER TABLE tts_scripts ADD COLUMN audio_path TEXT", []) {
        Ok(_) => println!("[dbStart] 迁移：已添加 audio_path 列到 tts_scripts"),
        Err(e) => println!("[dbStart] 迁移：tts_scripts.audio_path 列已存在 ({})", e),
    }

    Ok(())
}

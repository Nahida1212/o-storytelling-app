use std::{
    fs,
    path::PathBuf,
};

use crate::state::appState::AppState;
use crate::db::dbService;
use crate::tools::process_novel::{self, ProcessNovelResult};
use tauri::{AppHandle, Manager, State};
#[tauri::command]
pub fn file_upload(app: AppHandle, state: State<AppState>, file: String) -> Result<String, String> {
    println!("file {}", file);

    let mut book_path = PathBuf::new();
    book_path.push(file);

    let file_name = book_path.file_name().unwrap();

    let config_state = state.config.lock().unwrap().clone();


    println!("config path {}", config_state.novel_path.to_string_lossy());
    let mut to_path = config_state.novel_path.clone();
    to_path.push(file_name);
    println!("to_path {}" , to_path.to_string_lossy());


    // copy file
    if config_state.use_custom_dir {
        // use custom dir
        if config_state.novel_path.exists() {
            println!("use custom dir {}", config_state.novel_path.to_string_lossy());
            // path not null, copy file to path where setting
            fs::copy(&book_path, &to_path)
                .map_err(|e: std::io::Error| e.to_string())?;

            // processing the novel file
            let process_result = process_novel::process_novel(&to_path)
                .map_err(|e| e.to_string())?;

            println!("Processing completed, novel: {}, chapters: {}",
                     process_result.novel.title, process_result.chapters.len());

            // Save to database
            save_novel_to_database(&app, &process_result, &to_path, &config_state)
                .map_err(|e| format!("Failed to save to database: {}", e))?;

            println!("copy file to custom dir success {}", to_path.to_string_lossy());
        } else {
            // no set dir
            // use file original path
            println!("custom directory does not exist");
        }
    } else {
        // TODO: Handle case when not using custom directory
        println!("Not using custom directory");
    }

    Ok("upload file success".to_owned())
}

/// 保存插图到文件系统和数据库
fn save_illustrations(
    app: &AppHandle,
    novel_id: i64,
    illustrations: &[crate::tools::process_novel::ProcessedIllustration],
    config: &crate::config::appConfig::AppConfig,
) -> Result<(), String> {
    use std::fs;
    use std::path::Path;

    // 获取或创建图片目录
    let base_image_dir = if !config.image_path.as_os_str().is_empty() && config.image_path.exists() {
        config.image_path.clone()
    } else {
        // 使用系统图片目录下的项目文件夹
        let picture_dir = app.path()
            .picture_dir()
            .map_err(|e| format!("Failed to get picture directory: {}", e))?;
        let project_dir = picture_dir.join("o-storytelling-app");
        fs::create_dir_all(&project_dir)
            .map_err(|e| format!("Failed to create project image directory: {}", e))?;
        project_dir
    };

    // 创建小说特定文件夹（使用小说ID）
    let novel_image_dir = base_image_dir.join(format!("novel-{}", novel_id));
    fs::create_dir_all(&novel_image_dir)
        .map_err(|e| format!("Failed to create novel image directory: {}", e))?;

    println!("Saving illustrations to: {}", novel_image_dir.to_string_lossy());

    let mut illustration_data_list = Vec::new();

    for (index, illustration) in illustrations.iter().enumerate() {
        // 生成安全的文件名
        let fallback_name = format!("image_{}", index);
        let original_name = Path::new(&illustration.resource_name)
            .file_name()
            .unwrap_or_else(|| std::ffi::OsStr::new(&fallback_name));

        let mut file_name = original_name.to_string_lossy().to_string();
        // 清理文件名中的非法字符
        file_name = file_name.replace(|c: char| !c.is_alphanumeric() && c != '.' && c != '-', "_");

        // 确保有正确的文件扩展名
        let extension = match illustration.mime_type.as_str() {
            "image/jpeg" => "jpg",
            "image/jpg" => "jpg",
            "image/png" => "png",
            "image/gif" => "gif",
            "image/webp" => "webp",
            "image/svg+xml" => "svg",
            _ => {
                // 从原始文件名提取扩展名
                Path::new(&illustration.resource_name)
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .unwrap_or("bin")
            }
        };

        // 如果文件名没有扩展名，添加扩展名
        if !file_name.contains('.') {
            file_name = format!("{}.{}", file_name, extension);
        }

        let image_path = novel_image_dir.join(&file_name);

        // 保存图片文件
        fs::write(&image_path, &illustration.data)
            .map_err(|e| format!("Failed to write image file {}: {}", image_path.to_string_lossy(), e))?;

        println!("Saved illustration: {} ({} bytes)", image_path.to_string_lossy(), illustration.data.len());

        // 准备数据库记录
        let illustration_data = dbService::IllustrationData {
            id: None,
            novel_id,
            image_path: image_path.to_string_lossy().to_string(),
            description: Some(illustration.resource_name.clone()), // 使用原始资源名作为描述
            chapter_index: illustration.chapter_index,
        };

        illustration_data_list.push(illustration_data);
    }

    // 批量插入插图记录
    if !illustration_data_list.is_empty() {
        dbService::insert_illustrations_batch(app, &illustration_data_list)
            .map_err(|e| format!("Failed to insert illustrations to database: {}", e))?;
        println!("Inserted {} illustrations to database", illustration_data_list.len());
    }

    Ok(())
}

/// 提取并保存封面图片
/// 返回封面图片的保存路径（如果找到并保存成功）
fn extract_and_save_cover_image(
    app: &AppHandle,
    novel_id: i64,
    illustrations: &[crate::tools::process_novel::ProcessedIllustration],
    config: &crate::config::appConfig::AppConfig,
) -> Result<Option<String>, String> {
    use std::fs;
    use std::path::Path;

    if illustrations.is_empty() {
        return Ok(None);
    }

    // 尝试找到封面图片
    let cover_illustration = illustrations.iter().find(|ill| {
        let name_lower = ill.resource_name.to_lowercase();
        name_lower.contains("cover") || name_lower.contains("封面")
    });

    // 如果没有找到封面图片，使用第一个图片
    let illustration = match cover_illustration {
        Some(ill) => ill,
        None => &illustrations[0],
    };

    // 获取或创建图片目录（与save_illustrations相同）
    let base_image_dir = if !config.image_path.as_os_str().is_empty() && config.image_path.exists() {
        config.image_path.clone()
    } else {
        // 使用系统图片目录下的项目文件夹
        let picture_dir = app.path()
            .picture_dir()
            .map_err(|e| format!("Failed to get picture directory: {}", e))?;
        let project_dir = picture_dir.join("o-storytelling-app");
        fs::create_dir_all(&project_dir)
            .map_err(|e| format!("Failed to create project image directory: {}", e))?;
        project_dir
    };

    // 创建小说特定文件夹（使用小说ID）
    let novel_image_dir = base_image_dir.join(format!("novel-{}", novel_id));
    fs::create_dir_all(&novel_image_dir)
        .map_err(|e| format!("Failed to create novel image directory: {}", e))?;

    // 生成封面图片文件名
    let fallback_name = "cover";
    let original_name = Path::new(&illustration.resource_name)
        .file_stem() // 获取不带扩展名的文件名
        .unwrap_or_else(|| std::ffi::OsStr::new(&fallback_name));

    let mut file_name = original_name.to_string_lossy().to_string();
    // 清理文件名中的非法字符
    file_name = file_name.replace(|c: char| !c.is_alphanumeric() && c != '.' && c != '-', "_");

    // 确保有正确的文件扩展名
    let extension = match illustration.mime_type.as_str() {
        "image/jpeg" => "jpg",
        "image/jpg" => "jpg",
        "image/png" => "png",
        "image/gif" => "gif",
        "image/webp" => "webp",
        "image/svg+xml" => "svg",
        _ => {
            // 从原始文件名提取扩展名
            Path::new(&illustration.resource_name)
                .extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("jpg") // 默认jpg
        }
    };

    // 如果文件名没有扩展名，添加扩展名
    if !file_name.contains('.') {
        file_name = format!("{}.{}", file_name, extension);
    } else {
        // 如果有扩展名，确保是正确的扩展名（可选）
        // 这里简单处理，直接使用原文件名
    }

    let cover_image_path = novel_image_dir.join(&file_name);

    // 保存图片文件
    fs::write(&cover_image_path, &illustration.data)
        .map_err(|e| format!("Failed to write cover image file {}: {}", cover_image_path.to_string_lossy(), e))?;

    println!("Saved cover image: {} ({} bytes)", cover_image_path.to_string_lossy(), illustration.data.len());

    // 返回相对路径或绝对路径？返回绝对路径以便前端使用
    // 注意：前端可能需要通过Tauri的asset协议访问
    // 暂时返回绝对路径字符串
    Ok(Some(cover_image_path.to_string_lossy().to_string()))
}

/// 保存小说数据到数据库
fn save_novel_to_database(
    app: &AppHandle,
    process_result: &ProcessNovelResult,
    file_path: &PathBuf,
    config: &crate::config::appConfig::AppConfig,
) -> Result<(), String> {
    // 准备小说数据
    let novel_data = dbService::NovelData {
        id: None, // 新插入的小说，id由数据库生成
        title: process_result.novel.title.clone(),
        author: process_result.novel.author.clone(),
        file_path: file_path.clone(),
        cover_image: None, // 保留为None，我们使用cover_image_path
        cover_image_path: None, // 稍后更新
    };

    // 检查是否已存在相同文件路径的小说
    if let Ok(Some(existing_id)) = dbService::find_novel_by_file_path(app, file_path) {
        // 如果已存在，删除旧记录
        dbService::delete_novel(app, existing_id)
            .map_err(|e| format!("Failed to delete existing novel: {}", e))?;
        println!("Deleted existing novel with id: {}", existing_id);
    }

    // 插入小说
    let novel_id = dbService::insert_novel(app, &novel_data)
        .map_err(|e| format!("Failed to insert novel: {}", e))?;

    println!("Inserted novel with id: {}", novel_id);

    // 提取并保存封面图片
    if !process_result.illustrations.is_empty() {
        match extract_and_save_cover_image(app, novel_id, &process_result.illustrations, config) {
            Ok(Some(cover_image_path)) => {
                // 更新小说封面图片路径
                dbService::update_novel_cover_image_path(app, novel_id, Some(&cover_image_path))
                    .map_err(|e| format!("Failed to update novel cover image path: {}", e))?;
                println!("Updated novel cover image path: {}", cover_image_path);
            }
            Ok(None) => {
                println!("No cover image extracted");
            }
            Err(e) => {
                // 封面图片保存失败，但不影响整体流程，记录日志
                println!("Warning: Failed to extract cover image: {}", e);
            }
        }
    } else {
        println!("No illustrations found for cover image");
    }

    // 准备章节数据
    let mut chapter_data_list = Vec::new();
    let mut chapter_index = 0;

    for chapter in &process_result.chapters {
        let chapter_data = dbService::ChapterData {
            novel_id,
            chapter_index,
            title: chapter.title.clone(),
            content: Some(chapter.content.clone()),
            audio_path: None, // TODO: Add audio path when generated
        };
        chapter_data_list.push(chapter_data);
        chapter_index += 1;
    }

    // 批量插入章节
    if !chapter_data_list.is_empty() {
        dbService::insert_chapters_batch(app, &chapter_data_list)
            .map_err(|e| format!("Failed to insert chapters: {}", e))?;
        println!("Inserted {} chapters", chapter_data_list.len());
    }

    // 保存插图
    if !process_result.illustrations.is_empty() {
        save_illustrations(app, novel_id, &process_result.illustrations, config)
            .map_err(|e| format!("Failed to save illustrations: {}", e))?;
    }

    Ok(())
}

/// 用于前端显示的小说信息
#[derive(serde::Serialize, tauri_ts_generator::TS)]
#[ts(export)]
pub struct NovelInfo {
    pub id: i64,
    pub title: String,
    pub author: Option<String>,
    pub file_path: String,
    pub cover_image_path: Option<String>,
}

#[tauri::command]
pub fn get_all_books(app: AppHandle) -> Result<Vec<NovelInfo>, String> {
    let novels = dbService::get_all_novels(&app)
        .map_err(|e| format!("Failed to get novels: {}", e))?;

    let novel_infos: Vec<NovelInfo> = novels
        .into_iter()
        .map(|novel| {
            // 使用数据库中的实际id，如果id为None（不应该发生），使用0
            let id = novel.id.unwrap_or(0);
            NovelInfo {
                id,
                title: novel.title,
                author: novel.author,
                file_path: novel.file_path.to_string_lossy().to_string(),
                cover_image_path: novel.cover_image_path,
            }
        })
        .collect();

    Ok(novel_infos)
}

/// 用于前端显示的章节信息
#[derive(serde::Serialize, tauri_ts_generator::TS)]
#[ts(export)]
pub struct ChapterInfo {
    pub id: i32,
    pub title: String,
    pub index: i32,
}

#[tauri::command]
pub fn get_book_details(app: AppHandle, novel_id: i64) -> Result<Option<NovelInfo>, String> {
    let novel = dbService::get_novel_by_id(&app, novel_id)
        .map_err(|e| format!("Failed to get novel details: {}", e))?;

    match novel {
        Some(novel_data) => {
            let id = novel_data.id.unwrap_or(0);
            let novel_info = NovelInfo {
                id,
                title: novel_data.title,
                author: novel_data.author,
                file_path: novel_data.file_path.to_string_lossy().to_string(),
                cover_image_path: novel_data.cover_image_path,
            };
            Ok(Some(novel_info))
        }
        None => Ok(None),
    }
}

#[tauri::command]
pub fn get_book_chapters(app: AppHandle, novel_id: i64) -> Result<Vec<ChapterInfo>, String> {
    let chapters = dbService::get_chapters_by_novel_id(&app, novel_id)
        .map_err(|e| format!("Failed to get chapters: {}", e))?;

    let chapter_infos: Vec<ChapterInfo> = chapters
        .into_iter()
        .map(|chapter| {
            // 使用chapter_index作为id，或者可以生成唯一id
            // 注意：这里使用chapter_index作为id，因为章节表没有单独的id字段
            ChapterInfo {
                id: chapter.chapter_index,
                title: chapter.title,
                index: chapter.chapter_index,
            }
        })
        .collect();

    Ok(chapter_infos)
}

#[tauri::command]
pub fn delete_books(app: AppHandle, book_ids: Vec<i64>) -> Result<(), String> {
    use std::fs;

    for book_id in book_ids {
        // 先获取小说信息
        let novel_data = dbService::get_novel_by_id(&app, book_id)
            .map_err(|e| format!("Failed to get novel {}: {}", book_id, e))?;

        if let Some(novel) = novel_data {
            // 删除小说文件
            if novel.file_path.exists() {
                if let Err(e) = fs::remove_file(&novel.file_path) {
                    eprintln!("Warning: Failed to delete novel file {}: {}", novel.file_path.display(), e);
                    // 继续执行，不因为文件删除失败而终止
                } else {
                    println!("Deleted novel file: {}", novel.file_path.display());
                }
            }

            // 删除封面图片文件
            if let Some(cover_path) = &novel.cover_image_path {
                let cover_path = std::path::Path::new(cover_path);
                if cover_path.exists() {
                    if let Err(e) = fs::remove_file(cover_path) {
                        eprintln!("Warning: Failed to delete cover image {}: {}", cover_path.display(), e);
                    } else {
                        println!("Deleted cover image: {}", cover_path.display());
                    }
                }
            }

            // 获取并删除插图文件
            let illustrations = dbService::get_illustrations_by_novel_id(&app, book_id)
                .map_err(|e| format!("Failed to get illustrations for novel {}: {}", book_id, e))?;

            for illustration in &illustrations {
                let image_path = std::path::Path::new(&illustration.image_path);
                if image_path.exists() {
                    if let Err(e) = fs::remove_file(image_path) {
                        eprintln!("Warning: Failed to delete illustration {}: {}", image_path.display(), e);
                    } else {
                        println!("Deleted illustration: {}", image_path.display());
                    }
                }
            }

            // 尝试删除插图目录（如果为空）
            if !illustrations.is_empty() {
                // 获取插图目录
                if let Some(first_illustration) = illustrations.first() {
                    let image_path = std::path::Path::new(&first_illustration.image_path);
                    if let Some(parent_dir) = image_path.parent() {
                        // 检查目录是否为空
                        if let Ok(entries) = fs::read_dir(parent_dir) {
                            if entries.count() == 0 {
                                if let Err(e) = fs::remove_dir(parent_dir) {
                                    eprintln!("Warning: Failed to remove empty illustration directory {}: {}", parent_dir.display(), e);
                                } else {
                                    println!("Removed empty illustration directory: {}", parent_dir.display());
                                }
                            }
                        }
                    }
                }
            }
        }

        // 删除数据库记录（这会级联删除章节和插图记录）
        dbService::delete_novel(&app, book_id)
            .map_err(|e| format!("Failed to delete book {} from database: {}", book_id, e))?;
    }
    Ok(())
}

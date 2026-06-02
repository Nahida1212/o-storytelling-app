use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use crate::db::db_service;
use crate::state::app_state::AppState;
use crate::tools::process_novel::ProcessedIllustration;
use tauri::{AppHandle, Manager, State};
/// 保存插图到文件系统和数据库
fn save_illustrations(
    app: &AppHandle,
    novel_id: i64,
    illustrations: &[crate::tools::process_novel::ProcessedIllustration],
    config: &crate::config::app_config::AppConfig,
) -> Result<(), String> {
    use std::fs;
    use std::path::Path;

    let base_image_dir: PathBuf =
        if !config.image_path.as_os_str().is_empty() && config.image_path.exists() {
            config.image_path.clone()
        } else {
            let picture_dir = app
                .path()
                .picture_dir()
                .map_err(|e| format!("Failed to get picture directory: {}", e))?;
            let project_dir = picture_dir.join("o-storytelling-app");
            fs::create_dir_all(&project_dir)
                .map_err(|e| format!("Failed to create project image directory: {}", e))?;
            project_dir
        };

    let novel_image_dir = base_image_dir.join(format!("novel-{}", novel_id));
    fs::create_dir_all(&novel_image_dir)
        .map_err(|e| format!("Failed to create novel image directory: {}", e))?;

    println!(
        "Saving illustrations to: {}",
        novel_image_dir.to_string_lossy()
    );

    let mut illustration_data_list = Vec::new();

    for (index, illustration) in illustrations.iter().enumerate() {
        let fallback_name = format!("image_{}", index);
        let original_name = Path::new(&illustration.resource_name)
            .file_name()
            .unwrap_or_else(|| std::ffi::OsStr::new(&fallback_name));

        let mut file_name = original_name.to_string_lossy().to_string();
        file_name = file_name.replace(|c: char| !c.is_alphanumeric() && c != '.' && c != '-', "_");

        let extension = match illustration.mime_type.as_str() {
            "image/jpeg" => "jpg",
            "image/jpg" => "jpg",
            "image/png" => "png",
            "image/gif" => "gif",
            "image/webp" => "webp",
            "image/svg+xml" => "svg",
            _ => Path::new(&illustration.resource_name)
                .extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("bin"),
        };

        if !file_name.contains('.') {
            file_name = format!("{}.{}", file_name, extension);
        }

        let image_path = novel_image_dir.join(&file_name);

        fs::write(&image_path, &illustration.data).map_err(|e| {
            format!(
                "Failed to write image file {}: {}",
                image_path.to_string_lossy(),
                e
            )
        })?;

        println!(
            "Saved illustration: {} ({} bytes)",
            image_path.to_string_lossy(),
            illustration.data.len()
        );

        let illustration_data = db_service::IllustrationData {
            id: None,
            novel_id,
            image_path: image_path.to_string_lossy().to_string(),
            description: Some(illustration.resource_name.clone()),
            chapter_index: illustration.chapter_index,
        };

        illustration_data_list.push(illustration_data);
    }

    if !illustration_data_list.is_empty() {
        db_service::insert_illustrations_batch(app, &illustration_data_list)
            .map_err(|e| format!("Failed to insert illustrations to database: {}", e))?;
        println!(
            "Inserted {} illustrations to database",
            illustration_data_list.len()
        );
    }

    Ok(())
}

/// 提取并保存封面图片
/// 返回封面图片的保存路径（如果找到并保存成功）
fn extract_and_save_cover_image(
    app: &AppHandle,
    novel_id: i64,
    illustrations: &[crate::tools::process_novel::ProcessedIllustration],
    config: &crate::config::app_config::AppConfig,
) -> Result<Option<String>, String> {
    use std::fs;
    use std::path::Path;

    if illustrations.is_empty() {
        return Ok(None);
    }

    // 第一张图片即为封面
    let illustration = &illustrations[0];

    // 获取或创建图片目录（与save_illustrations相同）
    let base_image_dir: PathBuf =
        if !config.image_path.as_os_str().is_empty() && config.image_path.exists() {
            config.image_path.clone()
        } else {
            // 使用系统图片目录下的项目文件夹
            let picture_dir = app
                .path()
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
    fs::write(&cover_image_path, &illustration.data).map_err(|e| {
        format!(
            "Failed to write cover image file {}: {}",
            cover_image_path.to_string_lossy(),
            e
        )
    })?;

    println!(
        "Saved cover image: {} ({} bytes)",
        cover_image_path.to_string_lossy(),
        illustration.data.len()
    );

    // 返回相对路径或绝对路径？返回绝对路径以便前端使用
    // 注意：前端可能需要通过Tauri的asset协议访问
    // 暂时返回绝对路径字符串
    Ok(Some(cover_image_path.to_string_lossy().to_string()))
}

// /// 保存小说数据到数据库
// fn save_novel_to_database(
//     app: &AppHandle,
//     process_result: &ProcessNovelResult,
//     file_path: &PathBuf,
//     config: &crate::config::app_config::AppConfig,
// ) -> Result<(), String> {
//     // 准备小说数据
//     let novel_data = db_service::NovelData {
//         id: None, // 新插入的小说，id由数据库生成
//         title: process_result.novel.title.clone(),
//         author: process_result.novel.author.clone(),
//         file_path: file_path.clone(),
//         cover_image: None, // 保留为None，我们使用cover_image_path
//         cover_image_path: None, // 稍后更新
//     };

//     // 检查是否已存在相同文件路径的小说
//     if let Ok(Some(existing_id)) = db_service::find_novel_by_file_path(app, file_path) {
//         // 如果已存在，删除旧记录
//         db_service::delete_novel(app, existing_id)
//             .map_err(|e| format!("Failed to delete existing novel: {}", e))?;
//         println!("Deleted existing novel with id: {}", existing_id);
//     }

//     // 插入小说
//     let novel_id = db_service::insert_novel(app, &novel_data)
//         .map_err(|e| format!("Failed to insert novel: {}", e))?;

//     println!("Inserted novel with id: {}", novel_id);

//     // 提取并保存封面图片
//     if !process_result.illustrations.is_empty() {
//         match extract_and_save_cover_image(app, novel_id, &process_result.illustrations, config) {
//             Ok(Some(cover_image_path)) => {
//                 db_service::update_novel_cover_image_path(app, novel_id, Some(&cover_image_path))
//                     .map_err(|e| format!("Failed to update novel cover image path: {}", e))?;
//                 println!("Updated novel cover image path: {}", cover_image_path);
//             }
//             Ok(None) => {
//                 println!("No cover image extracted");
//             }
//             Err(e) => {
//                 println!("Warning: Failed to extract cover image: {}", e);
//             }
//         }
//     } else {
//         println!("No illustrations found for cover image");
//     }

//     // 准备章节数据
//     let mut chapter_data_list = Vec::new();
//     let mut chapter_index = 0;

//     for chapter in &process_result.chapters {
//         let chapter_data = db_service::ChapterData {
//             novel_id,
//             chapter_index,
//             title: chapter.title.clone(),
//             content: Some(chapter.content.clone()),
//             audio_path: None,
//             tts_generated: false,
//         };
//         chapter_data_list.push(chapter_data);
//         chapter_index += 1;
//     }

//     // 批量插入章节
//     if !chapter_data_list.is_empty() {
//         db_service::insert_chapters_batch(app, &chapter_data_list)
//             .map_err(|e| format!("Failed to insert chapters: {}", e))?;
//         println!("Inserted {} chapters", chapter_data_list.len());
//     }

//     // 保存插图
//     if !process_result.illustrations.is_empty() {
//         save_illustrations(app, novel_id, &process_result.illustrations, config)
//             .map_err(|e| format!("Failed to save illustrations: {}", e))?;
//     }

//     Ok(())
// }

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
    let novels =
        db_service::get_all_novels(&app).map_err(|e| format!("Failed to get novels: {}", e))?;

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
    pub content: Option<String>,
    pub tts_generated: bool,
}

#[tauri::command]
pub fn get_book_details(app: AppHandle, novel_id: i64) -> Result<Option<NovelInfo>, String> {
    let novel = db_service::get_novel_by_id(&app, novel_id)
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
    let chapters = db_service::get_chapters_by_novel_id(&app, novel_id)
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
                content: chapter.content,
                tts_generated: chapter.tts_generated,
            }
        })
        .collect();

    Ok(chapter_infos)
}

#[tauri::command]
pub fn get_chapter_content(
    app: AppHandle,
    novel_id: i64,
    chapter_index: i32,
) -> Result<Option<ChapterInfo>, String> {
    let chapter = db_service::get_chapter_by_index(&app, novel_id, chapter_index)
        .map_err(|e| format!("获取章节内容失败: {}", e))?;

    Ok(chapter.map(|c| ChapterInfo {
        id: c.chapter_index,
        title: c.title,
        index: c.chapter_index,
        content: c.content,
        tts_generated: c.tts_generated,
    }))
}

#[tauri::command]
pub fn delete_books(app: AppHandle, book_ids: Vec<i64>) -> Result<(), String> {
    use std::fs;

    for book_id in book_ids {
        // 先获取小说信息
        let novel_data = db_service::get_novel_by_id(&app, book_id)
            .map_err(|e| format!("Failed to get novel {}: {}", book_id, e))?;

        if let Some(novel) = novel_data {
            // 删除小说文件
            if novel.file_path.exists() {
                if let Err(e) = fs::remove_file(&novel.file_path) {
                    eprintln!(
                        "Warning: Failed to delete novel file {}: {}",
                        novel.file_path.display(),
                        e
                    );
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
                        eprintln!(
                            "Warning: Failed to delete cover image {}: {}",
                            cover_path.display(),
                            e
                        );
                    } else {
                        println!("Deleted cover image: {}", cover_path.display());
                    }
                }
            }

            // 获取并删除插图文件
            let illustrations = db_service::get_illustrations_by_novel_id(&app, book_id)
                .map_err(|e| format!("Failed to get illustrations for novel {}: {}", book_id, e))?;

            for illustration in &illustrations {
                let image_path = std::path::Path::new(&illustration.image_path);
                if image_path.exists() {
                    if let Err(e) = fs::remove_file(image_path) {
                        eprintln!(
                            "Warning: Failed to delete illustration {}: {}",
                            image_path.display(),
                            e
                        );
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
                                    println!(
                                        "Removed empty illustration directory: {}",
                                        parent_dir.display()
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }

        // 删除数据库记录（这会级联删除章节和插图记录）
        db_service::delete_novel(&app, book_id)
            .map_err(|e| format!("Failed to delete book {} from database: {}", book_id, e))?;
    }
    Ok(())
}

// ============================================================
// 新的导入流程（前端 epubjs 解析 + 后端存储）
// ============================================================

/// 前端解析后的章节数据
#[derive(serde::Deserialize)]
pub struct ImportChapterData {
    pub title: String,
    pub content: String,
}

/// 导入书籍：接收前端 epubjs 解析后的结构化数据
/// - 前端已处理：EPUB 解析、章节提取
/// - 后端负责：文件复制、数据库写入
#[tauri::command]
pub fn import_book(
    app: AppHandle,
    state: State<AppState>,
    source_path: String,
    title: String,
    author: Option<String>,
    #[allow(unused_variables)] publisher: Option<String>,
    #[allow(unused_variables)] description: Option<String>,
    chapters: Vec<ImportChapterData>,
) -> Result<NovelInfo, String> {
    let config = state.config.lock().unwrap().clone();

    // 1. 复制文件到书籍目录
    let file_name = Path::new(&source_path)
        .file_name()
        .ok_or_else(|| "Invalid file path".to_string())?;
    let mut dest_path = config.novel_path.clone();
    dest_path.push(file_name);

    fs::copy(&source_path, &dest_path).map_err(|e| format!("Failed to copy file: {}", e))?;
    println!("Copied file to: {}", dest_path.to_string_lossy());

    // 2. 检查是否已存在相同文件
    if let Ok(Some(existing_id)) = db_service::find_novel_by_file_path(&app, &dest_path) {
        db_service::delete_novel(&app, existing_id)
            .map_err(|e| format!("Failed to delete existing novel: {}", e))?;
        println!("Deleted existing novel with id: {}", existing_id);
    }

    // 3. 插入小说
    let novel_data = db_service::NovelData {
        id: None,
        title: title.clone(),
        author: author.clone(),
        file_path: dest_path.clone(),
        cover_image: None,
        cover_image_path: None,
    };
    let novel_id = db_service::insert_novel(&app, &novel_data)
        .map_err(|e| format!("Failed to insert novel: {}", e))?;
    println!("Inserted novel with id: {}", novel_id);

    // 4. 插入章节
    let chapter_data_list: Vec<db_service::ChapterData> = chapters
        .iter()
        .enumerate()
        .map(|(i, ch)| db_service::ChapterData {
            novel_id,
            chapter_index: i as i32,
            title: ch.title.clone(),
            content: Some(ch.content.clone()),
            audio_path: None,
            tts_generated: false,
        })
        .collect();

    if !chapter_data_list.is_empty() {
        db_service::insert_chapters_batch(&app, &chapter_data_list)
            .map_err(|e| format!("Failed to insert chapters: {}", e))?;
        println!("Inserted {} chapters", chapter_data_list.len());
    }

    // 5. 从 EPUB 提取所有图片
    let illustrations = extract_all_images_from_epub(&dest_path)?;
    let mut cover_image_path: Option<String> = None;

    if !illustrations.is_empty() {
        // 保存封面
        match extract_and_save_cover_image(&app, novel_id, &illustrations, &config) {
            Ok(Some(path)) => {
                db_service::update_novel_cover_image_path(&app, novel_id, Some(&path))
                    .map_err(|e| format!("Failed to update cover path: {}", e))?;
                println!("Saved cover image: {}", path);
                cover_image_path = Some(path);
            }
            Ok(None) => {}
            Err(e) => eprintln!("Warning: Failed to save cover: {}", e),
        }
        // 保存插图
        save_illustrations(&app, novel_id, &illustrations, &config)?;
    }

    Ok(NovelInfo {
        id: novel_id,
        title,
        author,
        file_path: dest_path.to_string_lossy().to_string(),
        cover_image_path,
    })
}

/// 从 EPUB 文件中提取所有图片资源
fn extract_all_images_from_epub(epub_path: &Path) -> Result<Vec<ProcessedIllustration>, String> {
    let mut epub_doc = epub::doc::EpubDoc::new(epub_path)
        .map_err(|e| format!("Failed to open EPUB for images: {:?}", e))?;

    let mut illustrations = Vec::new();
    let mut seen_names: HashSet<String> = HashSet::new();

    // 1. 优先提取封面（使用内置 get_cover）
    if let Some((data, mime)) = epub_doc.get_cover() {
        seen_names.insert("cover".to_string());
        illustrations.push(ProcessedIllustration {
            resource_name: "cover".to_string(),
            mime_type: mime,
            data,
            chapter_index: None,
        });
        println!("[extract_images] cover image extracted");
    }

    // 提前克隆 spine 和 resources 数据，避免借用冲突
    let spine_idrefs: Vec<(usize, String)> = epub_doc
        .spine
        .iter()
        .enumerate()
        .map(|(i, item)| (i, item.idref.clone()))
        .collect();
    let resource_ids: Vec<String> = epub_doc.resources.keys().cloned().collect();

    // 2. 遍历 spine，提取每个页面 HTML 中的 <img> 引用
    for (i, idref) in &spine_idrefs {
        let i = *i;
        if let Some((html, _)) = epub_doc.get_resource_str(idref) {
            for src in extract_img_srcs(&html) {
                let fname = Path::new(&src)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("");
                if fname.is_empty() || seen_names.contains(fname) {
                    continue;
                }
                seen_names.insert(fname.to_string());

                // 按路径获取图片数据
                if let Some(data) = epub_doc.get_resource_by_path(&src) {
                    let mime = epub_doc
                        .get_resource_mime_by_path(&src)
                        .unwrap_or_else(|| "image/jpeg".to_string());
                    illustrations.push(ProcessedIllustration {
                        resource_name: fname.to_string(),
                        mime_type: mime,
                        data,
                        chapter_index: Some(i as i32),
                    });
                    println!("[extract_images]   from spine[{}] by path: '{}'", i, src);
                } else if let Some((data, mime)) = epub_doc.get_resource(&src) {
                    // 按资源 id 回退
                    illustrations.push(ProcessedIllustration {
                        resource_name: fname.to_string(),
                        mime_type: mime,
                        data,
                        chapter_index: Some(i as i32),
                    });
                    println!("[extract_images]   from spine[{}] by id: '{}'", i, src);
                } else {
                    println!("[extract_images]   not found: '{}'", src);
                }
            }
        }
    }

    // 3. 扫描 resources 中未被覆盖的 image/* 类型资源
    for id in &resource_ids {
        if seen_names.contains(id) {
            continue;
        }
        let is_image = epub_doc
            .resources
            .get(id)
            .map(|item| item.mime.starts_with("image/"))
            .unwrap_or(false);
        if is_image {
            if let Some((data, mime)) = epub_doc.get_resource(id) {
                let resource_name = Path::new(id)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(id)
                    .to_string();
                illustrations.push(ProcessedIllustration {
                    resource_name,
                    mime_type: mime,
                    data,
                    chapter_index: None,
                });
                seen_names.insert(id.clone());
                println!("[extract_images]   direct resource: '{}'", id);
            }
        }
    }

    println!(
        "[extract_images] total images extracted: {}",
        illustrations.len()
    );
    Ok(illustrations)
}

/// 从 HTML 中提取 <img src="...">（跳过 data: 内联图片）
fn extract_img_srcs(html: &str) -> Vec<String> {
    let mut srcs = Vec::new();
    let lower = html.to_lowercase();
    let mut pos = 0;
    while let Some(img_start) = lower[pos..].find("<img") {
        let abs_start = pos + img_start;
        let tag_end = match lower[abs_start..].find('>') {
            Some(end) => abs_start + end,
            None => break,
        };
        let tag = &lower[abs_start..=tag_end];
        if let Some(src_start) = tag.find("src=\"") {
            let start = src_start + 5;
            if let Some(src_end) = tag[start..].find('"') {
                let src = &html[abs_start + start..abs_start + start + src_end];
                if !src.starts_with("data:") {
                    srcs.push(clean_img_path(src));
                }
            }
        }
        pos = tag_end + 1;
    }
    srcs
}

/// 清理图片路径（去掉 ../ 前缀）
fn clean_img_path(src: &str) -> String {
    if src.starts_with("http://") || src.starts_with("https://") {
        return src.to_string();
    }
    let parts: Vec<&str> = src.split('/').filter(|p| *p != "..").collect();
    parts.join("/")
}

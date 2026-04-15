use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tauri_ts_generator::TS;

/// 章节类型枚举
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ChapterType {
    /// 封面
    Cover,
    /// 标题页
    TitlePage,
    /// 目录
    Toc,
    /// 前言/序言
    FrontMatter,
    /// 正文章节
    ChapterBody,
    /// 版权页
    Copyright,
    /// 结语
    BackMatter,
    /// 未知类型
    Unknown,
}

impl Default for ChapterType {
    fn default() -> Self {
        Self::Unknown
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct Novel {
    id: String,
    auhter: String,
    title: String,
    progress: f32,
    introduction: String,
    path: PathBuf,
    pubisher: String,
}

impl Default for Novel {
    fn default() -> Self {
        Self {
            id: String::new(),
            auhter: String::new(),
            title: String::new(),
            progress: 0.0,
            introduction: String::new(),
            path: PathBuf::new(),
            pubisher: String::new(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize, TS, Debug)]
pub struct Chapter {
    id: String,
    book_id: String,
    title: String,
    index: i32,
    content: String,
    word_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    chapter_type: Option<ChapterType>,
}

impl Default for Chapter {
    fn default() -> Self {
        Self {
            id: String::new(),
            book_id: String::new(),
            title: String::new(),
            index: 0,
            content: String::new(),
            word_count: 0,
            chapter_type: None,
        }
    }
}

/// 处理后的novel数据，用于保存到数据库
#[derive(Debug)]
pub struct ProcessedNovel {
    pub title: String,
    pub author: Option<String>,
    pub file_path: PathBuf,
    pub publisher: Option<String>,
    pub introduction: Option<String>,
}

/// 处理后的chapter数据，用于保存到数据库
#[derive(Debug)]
pub struct ProcessedChapter {
    pub chapter_index: i32,
    pub title: String,
    pub content: String,
    pub chapter_type: ChapterType,
    pub word_count: usize,
}

/// 处理后的插图数据，用于保存到数据库
#[derive(Debug)]
pub struct ProcessedIllustration {
    pub resource_name: String, // 资源名称（如 "cover.jpg"）
    pub mime_type: String,     // MIME类型（如 "image/jpeg"）
    pub data: Vec<u8>,         // 图片数据
    pub chapter_index: Option<i32>, // 关联的章节索引（如果有）
}

/// 处理epub文件的结果
#[derive(Debug)]
pub struct ProcessNovelResult {
    pub novel: ProcessedNovel,
    pub chapters: Vec<ProcessedChapter>,
    pub illustrations: Vec<ProcessedIllustration>,
}

pub fn process_novel(path: &PathBuf) -> Result<ProcessNovelResult, String> {
    println!("进入process novel函数，路径: {}", path.to_string_lossy());
    // 打开 epub 文件
    let mut epub_doc: epub::doc::EpubDoc<std::io::BufReader<std::fs::File>> = epub::doc::EpubDoc::new(path)
        .map_err(|e| format!("Failed to open epub file: {:?}", e))?;

    // 读取元数据
    println!("========== EPUB 元数据 ==========");

    // 提取元数据
    let title = epub_doc.mdata("title").map(|m| m.value.clone()).unwrap_or_default();
    let author = epub_doc.mdata("creator").map(|m| m.value.clone());
    let publisher = epub_doc.mdata("publisher").map(|m| m.value.clone());
    let introduction = epub_doc.mdata("description").map(|m| m.value.clone());

    // 打印元数据
    println!("标题: {}", title);
    println!("作者: {:?}", author);
    println!("出版社: {:?}", publisher);
    println!("简介: {:?}", introduction);

    // 获取所有元数据
    println!("\n---------- 所有元数据字段 ----------");
    for item in epub_doc.metadata.iter() {
        println!("{}: {} (lang: {:?}, refined: {:?})", item.property, item.value, item.lang, item.refined);
    }

    // 提取插图资源
    println!("\n========== 提取插图 ==========");
    let mut illustrations = Vec::new();
    // 首先收集所有图片资源的名称
    let image_names: Vec<String> = epub_doc.resources.keys()
        .filter(|name| {
            let mime_type = epub_doc.get_resource_mime(name).unwrap_or_default();
            mime_type.starts_with("image/")
        })
        .cloned()
        .collect();

    println!("发现 {} 个图片资源", image_names.len());

    for name in image_names {
        let mime_type = epub_doc.get_resource_mime(&name).unwrap_or_default();
        if let Some((data, _actual_mime)) = epub_doc.get_resource(&name) {
            println!("提取插图: {} ({}), 大小: {} 字节", name, mime_type, data.len());
            illustrations.push(ProcessedIllustration {
                resource_name: name.clone(),
                mime_type: mime_type.clone(),
                data,
                chapter_index: None, // 暂时不关联章节，后续可以改进
            });
        }
    }
    println!("共提取 {} 个插图", illustrations.len());

    // 读取章节信息
    println!("\n========== 章节信息 ==========");
    println!("总章节数: {}", epub_doc.spine.len());

    // 用于判断章节位置（前几个通常是封面、目录等）
    let total_spine = epub_doc.spine.len();

    // 先收集所有 spine idref 和 properties
    let spine_items: Vec<(String, Option<String>)> = epub_doc.spine.iter()
        .map(|item| (item.idref.clone(), item.properties.clone()))
        .collect();

    let mut chapters = Vec::new();
    let mut chapter_count = 0;
    let mut front_matter_count = 0;
    let mut back_matter_count = 0;

    for (index, (spine_id, properties)) in spine_items.iter().enumerate() {
        println!("\n--- 章节 {} ---", index + 1);
        println!("Spine ID: {}", spine_id);
        println!("Spine Properties: {:?}", properties);

        // 尝试获取章节内容
        let content_result = epub_doc.get_resource_str(spine_id);
        if let Some((content, mime)) = content_result {
            println!("MIME类型: {}", mime);

            // 提取 epub:type 属性
            let epub_type = extract_epub_type(&content);
            println!("提取到的 epub:type: \"{}\"", epub_type);

            // 提取章节标题
            let chapter_title = extract_chapter_title(&content);
            println!("章节标题: {}", chapter_title);

            // 分析 HTML 结构
            let (a_count, has_img, text_length) = analyze_html_structure(&content);
            println!("结构分析: <a>标签数={}, 有图片={}, 文本长度={}", a_count, has_img, text_length);

            // 提取 body class
            let body_class = extract_body_class(&content);
            println!("Body class: \"{}\"", body_class);

            // 分类章节类型（按优先级：epub:type → 结构 → class）
            let chapter_type = classify_chapter_type(
                &epub_type,
                spine_id,
                &chapter_title,
                index,
                total_spine,
                text_length,
                properties,
                a_count,
                has_img,
                &body_class
            );
            println!("章节类型: {:?}", chapter_type);

            // 调试：打印分类的关键因素
            println!("分类决策因素: index={}, total_spine={}, text_length={}, title='{}', epub_type='{}', spine_id='{}'",
                     index, total_spine, text_length, chapter_title, epub_type, spine_id);

            // 根据类型统计
            match chapter_type {
                ChapterType::ChapterBody => chapter_count += 1,
                ChapterType::FrontMatter => front_matter_count += 1,
                ChapterType::BackMatter => back_matter_count += 1,
                _ => {}
            }

            // 提取纯文本内容（用于数据库存储）
            let text_content = extract_text_from_html(&content);
            let word_count = text_content.chars().count();
            println!("提取的文本长度: {} 字符", word_count);

            // 只保存正文章节、前言和结语，跳过封面、目录等非内容章节
            let should_save = match chapter_type {
                ChapterType::ChapterBody | ChapterType::FrontMatter | ChapterType::BackMatter => true,
                _ => false,
            };

            if should_save {
                let processed_chapter = ProcessedChapter {
                    chapter_index: index as i32,
                    title: chapter_title,
                    content: text_content,
                    chapter_type,
                    word_count,
                };
                chapters.push(processed_chapter);
            }
        } else {
            println!("无法读取章节内容");
        }
    }

    println!("\n========== 章节统计 ==========");
    println!("正文章节数: {}", chapter_count);
    println!("前言/序言数: {}", front_matter_count);
    println!("结语数: {}", back_matter_count);
    println!("保存的章节数: {}", chapters.len());

    // 构建处理结果
    let processed_novel = ProcessedNovel {
        title,
        author,
        file_path: path.clone(),
        publisher,
        introduction,
    };

    Ok(ProcessNovelResult {
        novel: processed_novel,
        chapters,
        illustrations,
    })
}

/// 从HTML中提取纯文本内容
fn extract_text_from_html(html: &str) -> String {
    // 首先尝试使用html2text
    match html2text::from_read(html.as_bytes(), usize::MAX) {
        Ok(text) => {
            let trimmed = text.trim();
            if !trimmed.is_empty() && trimmed.chars().count() > 10 {
                return trimmed.to_string();
            }
        }
        Err(_) => {}
    }

    // 备用方法：简单去除HTML标签
    let mut in_tag = false;
    let mut result = String::new();
    let mut last_char_was_space = false;

    for c in html.chars() {
        if c == '<' {
            in_tag = true;
        } else if c == '>' {
            in_tag = false;
        } else if !in_tag {
            // 如果是空白字符，只添加一个空格
            if c.is_whitespace() {
                if !last_char_was_space && !result.is_empty() {
                    result.push(' ');
                    last_char_was_space = true;
                }
            } else {
                result.push(c);
                last_char_was_space = false;
            }
        }
    }

    result.trim().to_string()
}

/// 分析 HTML 结构
/// 返回：(a标签数量, 是否有img标签, 文本长度)
fn analyze_html_structure(html: &str) -> (usize, bool, usize) {
    // 统计 <a> 标签数量
    let a_count = html.matches("<a").count();

    // 检查是否有 <img 标签
    let has_img = html.contains("<img");

    // 获取纯文本长度
    let text_content = extract_text_from_html(html);
    let text_length = text_content.chars().count();

    (a_count, has_img, text_length)
}

/// 从 HTML 内容中提取 body 标签的 class 属性
fn extract_body_class(html: &str) -> String {
    // 查找 <body 标签
    if let Some(body_start) = html.find("<body") {
        let remaining = &html[body_start..];
        if let Some(tag_end_pos) = remaining.find('>') {
            let full_tag = &remaining[..tag_end_pos + 1];

            // 查找 class 属性
            if let Some(class_pos) = full_tag.find("class") {
                let after_class = &full_tag[class_pos + 5..]; // "class" 长度为 5

                // 跳过空格和等号
                let mut value_start = 0;
                for (i, c) in after_class.chars().enumerate() {
                    if c == '"' || c == '\'' || (c != ' ' && c != '=' && c != '\t' && c != '\n') {
                        value_start = i;
                        break;
                    }
                }

                let value_part = &after_class[value_start..];

                // 找到属性值结束位置
                let value_end = value_part.find(|c: char| c == '"' || c == '\'' || c == ' ' || c == '\t' || c == '\n');

                if let Some(end_pos) = value_end {
                    return value_part[..end_pos].to_string();
                } else if !value_part.is_empty() {
                    let result = value_part.trim_end_matches('>');
                    return result.to_string();
                }
            }
        }
    }

    String::new()
}

/// 从 HTML 内容中提取章节标题
fn extract_chapter_title(html: &str) -> String {
    // 首先提取纯文本内容
    let text_content = extract_text_from_html(html);
    let first_line = text_content.lines().next().unwrap_or("").trim();

    // 检查第一行是否适合作为章节标题
    // 规则1: 非空且长度适中（通常章节标题不会超过200字符）
    if !first_line.is_empty() && first_line.len() < 200 {
        // 规则2: 检查是否包含常见的章节标识符
        let line_lower = first_line.to_lowercase();
        let has_chapter_marker = line_lower.contains("第") && (line_lower.contains("章") || line_lower.contains("节") || line_lower.contains("回")) ||
                                line_lower.contains("chapter") || line_lower.contains("section") ||
                                line_lower.contains("part") || line_lower.contains("episode");

        // 规则3: 检查是否以数字开头（如 "1. " 或 "第1章"）
        let starts_with_number = first_line.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false);

        // 规则4: 检查是否包含数字加点格式（如 "1. 标题"）
        let has_number_dot_format = first_line.contains('.') &&
                                   first_line.split('.').next().unwrap_or("").trim().chars().all(|c| c.is_ascii_digit());

        // 如果满足任何章节标题的特征，使用第一行作为标题
        if has_chapter_marker || starts_with_number || has_number_dot_format {
            return first_line.to_string();
        }

        // 规则5: 如果第一行较短（小于80字符），也可能是一个标题
        if first_line.len() < 80 {
            return first_line.to_string();
        }
    }

    // 如果第一行不合适，尝试提取 HTML 标签中的标题
    // 尝试提取 <title> 标签内容
    if let Some(start) = html.find("<title>") {
        if let Some(end) = html.find("</title>") {
            let title = &html[start + 7..end];
            let trimmed = title.trim();
            if !trimmed.is_empty() {
                return trimmed.to_string();
            }
        }
    }

    // 尝试提取 <h1> 标签内容
    if let Some(start) = html.find("<h1") {
        if let Some(tag_end) = html[start..].find('>') {
            let h1_content_start = start + tag_end + 1;
            if let Some(end) = html[h1_content_start..].find("</h1>") {
                let title = &html[h1_content_start..h1_content_start + end];
                let trimmed = title.trim();
                if !trimmed.is_empty() {
                    return trimmed.to_string();
                }
            }
        }
    }

    // 尝试提取 <h2> 标签内容
    if let Some(start) = html.find("<h2") {
        if let Some(tag_end) = html[start..].find('>') {
            let h2_content_start = start + tag_end + 1;
            if let Some(end) = html[h2_content_start..].find("</h2>") {
                let title = &html[h2_content_start..h2_content_start + end];
                let trimmed = title.trim();
                if !trimmed.is_empty() {
                    return trimmed.to_string();
                }
            }
        }
    }

    // 尝试提取 <h3> 标签内容
    if let Some(start) = html.find("<h3") {
        if let Some(tag_end) = html[start..].find('>') {
            let h3_content_start = start + tag_end + 1;
            if let Some(end) = html[h3_content_start..].find("</h3>") {
                let title = &html[h3_content_start..h3_content_start + end];
                let trimmed = title.trim();
                if !trimmed.is_empty() {
                    return trimmed.to_string();
                }
            }
        }
    }

    // 尝试提取 <h4> 标签内容
    if let Some(start) = html.find("<h4") {
        if let Some(tag_end) = html[start..].find('>') {
            let h4_content_start = start + tag_end + 1;
            if let Some(end) = html[h4_content_start..].find("</h4>") {
                let title = &html[h4_content_start..h4_content_start + end];
                let trimmed = title.trim();
                if !trimmed.is_empty() {
                    return trimmed.to_string();
                }
            }
        }
    }

    // 最后，如果所有方法都失败，使用第一行（即使它可能很长）
    if !first_line.is_empty() {
        return first_line.to_string();
    }

    "未知章节".to_string()
}

/// 从 HTML 内容中提取 epub:type 属性
fn extract_epub_type(html: &str) -> String {
    // 尝试在多个标签中查找 epub:type
    let tags_to_check = ["<body", "<section", "<article", "<div", "<nav"];

    for tag in tags_to_check {
        if let Some(tag_start) = html.find(tag) {
            // 找到整个标签（从开始到 >）
            let remaining = &html[tag_start..];
            if let Some(tag_end_pos) = remaining.find('>') {
                let full_tag = &remaining[..tag_end_pos + 1];

                // 查找 epub:type 属性
                // 支持多种格式：epub:type="value", epub:type='value', epub:type=value
                if let Some(type_pos) = full_tag.find("epub:type") {
                    let after_type = &full_tag[type_pos + 10..]; // "epub:type" 长度为 10

                    // 跳过空格和等号
                    let mut value_start = 0;
                    for (i, c) in after_type.chars().enumerate() {
                        if c == '"' || c == '\'' || (c != ' ' && c != '=' && c != '\t' && c != '\n') {
                            value_start = i;
                            break;
                        }
                    }

                    let value_part = &after_type[value_start..];

                    // 找到属性值结束位置
                    let value_end = value_part.find(|c: char| c == '"' || c == '\'' || c == ' ' || c == '\t' || c == '\n');

                    if let Some(end_pos) = value_end {
                        return value_part[..end_pos].to_string();
                    } else if !value_part.is_empty() {
                        // 如果没有结束符但值不为空，返回剩余部分（去掉最后的 >）
                        let result = value_part.trim_end_matches('>');
                        return result.to_string();
                    }
                }
            }
        }
    }

    String::new()
}

/// 按优先级分类章节类型
/// 第一优先：epub:type
/// 第二优先：spine_id 解析（根据命名规则）
fn classify_chapter_type(
    epub_type: &str,
    spine_id: &str,
    title: &str,
    _index: usize,
    total_spine: usize,
    text_length: usize,
    _spine_properties: &Option<String>,
    _a_count: usize,
    _has_img: bool,
    _body_class: &str
) -> ChapterType {
    println!("分类函数调用: spine_id='{}', epub_type='{}', title='{}', text_length={}",
             spine_id, epub_type, title, text_length);

    // ===== 第一优先：epub:type =====
    let epub_type_lower = epub_type.to_lowercase();

    if !epub_type_lower.is_empty() {
        if epub_type_lower.contains("cover") {
            println!("  基于epub_type分类为: Cover (包含'cover')");
            return ChapterType::Cover;
        }
        if epub_type_lower.contains("titlepage") {
            println!("  基于epub_type分类为: TitlePage (包含'titlepage')");
            return ChapterType::TitlePage;
        }
        if epub_type_lower.contains("toc") || epub_type_lower.contains("nav") {
            println!("  基于epub_type分类为: Toc (包含'toc'或'nav')");
            return ChapterType::Toc;
        }
        if epub_type_lower.contains("frontmatter") || epub_type_lower.contains("dedication")
            || epub_type_lower.contains("foreword") || epub_type_lower.contains("preface")
            || epub_type_lower.contains("acknowledgments") || epub_type_lower.contains("prologue") {
            println!("  基于epub_type分类为: FrontMatter (包含相关关键词)");
            return ChapterType::FrontMatter;
        }
        if epub_type_lower.contains("backmatter") || epub_type_lower.contains("afterword")
            || epub_type_lower.contains("appendix") || epub_type_lower.contains("glossary")
            || epub_type_lower.contains("index") || epub_type_lower.contains("colophon")
            || epub_type_lower.contains("epilogue") {
            println!("  基于epub_type分类为: BackMatter (包含相关关键词)");
            return ChapterType::BackMatter;
        }
        if epub_type_lower.contains("copyright") || epub_type_lower.contains("copyright-page") {
            println!("  基于epub_type分类为: Copyright (包含'copyright')");
            return ChapterType::Copyright;
        }
        if epub_type_lower.contains("chapter") || epub_type_lower.contains("bodymatter") {
            println!("  基于epub_type分类为: ChapterBody (包含'chapter'或'bodymatter')");
            return ChapterType::ChapterBody;
        }
    }

    // ===== 第二优先：spine_id 解析 =====
    let spine_id_lower = spine_id.to_lowercase();

    // 检查 spine_id 中的类型标识
    // 注意：spine_id 可能是类似 "p-037", "p-cover", "BackMatter" 等格式

    // 1. 检查封面相关
    if spine_id_lower.contains("cover") {
        println!("  基于spine_id分类为: Cover (包含'cover')");
        return ChapterType::Cover;
    }

    // 2. 检查标题页
    if spine_id_lower.contains("titlepage") || spine_id_lower.contains("title-page") {
        println!("  基于spine_id分类为: TitlePage (包含'titlepage'或'title-page')");
        return ChapterType::TitlePage;
    }

    // 3. 检查目录
    if spine_id_lower.contains("toc") || spine_id_lower.contains("contents") || spine_id_lower.contains("nav") {
        println!("  基于spine_id分类为: Toc (包含'toc', 'contents'或'nav')");
        return ChapterType::Toc;
    }

    // 4. 检查版权页
    if spine_id_lower.contains("copyright") || spine_id_lower.contains("colophon") || spine_id_lower.contains("rights") {
        println!("  基于spine_id分类为: Copyright (包含'copyright', 'colophon'或'rights')");
        return ChapterType::Copyright;
    }

    // 5. 检查前言/序言 (FrontMatter)
    if spine_id_lower.contains("frontmatter") || spine_id_lower.contains("fmatter")
        || spine_id_lower.contains("preface") || spine_id_lower.contains("foreword")
        || spine_id_lower.contains("prologue") || spine_id_lower.contains("dedication")
        || spine_id_lower.contains("acknowledgment") || spine_id_lower.contains("intro") {
        println!("  基于spine_id分类为: FrontMatter (包含相关关键词)");
        return ChapterType::FrontMatter;
    }

    // 6. 检查结语/后记 (BackMatter)
    if spine_id_lower.contains("backmatter") || spine_id_lower.contains("epilogue")
        || spine_id_lower.contains("afterword") || spine_id_lower.contains("appendix")
        || spine_id_lower.contains("glossary") || spine_id_lower.contains("index")
        || spine_id_lower == "backmatter" || spine_id_lower == "colophon" {
        println!("  基于spine_id分类为: BackMatter (包含相关关键词或完全匹配)");
        return ChapterType::BackMatter;
    }

    // 7. 检查正文章节 - 基于命名模式
    // 模式1: "p-" 后跟数字 (如 "p-037", "p-5")
    if spine_id_lower.starts_with("p-") {
        let after_prefix = &spine_id_lower[2..]; // 去掉 "p-"
        // 检查是否是纯数字
        if after_prefix.chars().all(|c| c.is_ascii_digit()) {
            println!("  基于spine_id分类为: ChapterBody (p-后跟纯数字: '{}')", after_prefix);
            return ChapterType::ChapterBody;
        }
        // 或者检查是否以数字开头 (如 "p-5-something")
        if after_prefix.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
            println!("  基于spine_id分类为: ChapterBody (p-后以数字开头: '{}')", after_prefix);
            return ChapterType::ChapterBody;
        }
    }

    // 模式2: 包含 "chapter" 或 "ch"
    if spine_id_lower.contains("chapter") || spine_id_lower.contains("ch-") {
        println!("  基于spine_id分类为: ChapterBody (包含'chapter'或'ch-')");
        return ChapterType::ChapterBody;
    }

    // 模式3: 直接是数字或包含数字
    if spine_id_lower.chars().any(|c| c.is_ascii_digit()) {
        // 但排除广告等特殊情况
        if !spine_id_lower.contains("ad-") && !spine_id_lower.contains("advertisement") {
            println!("  基于spine_id分类为: ChapterBody (包含数字且不是广告)");
            return ChapterType::ChapterBody;
        }
    }

    // 模式4: 包含数字的文件名 (如 "037.xhtml", "005.html")
    if spine_id_lower.ends_with(".xhtml") || spine_id_lower.ends_with(".html") {
        let filename = spine_id_lower.split('/').last().unwrap_or(&spine_id_lower);
        if filename.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
            println!("  基于spine_id分类为: ChapterBody (数字开头的HTML文件: '{}')", filename);
            return ChapterType::ChapterBody;
        }
    }

    // ===== 小型EPUB特殊处理 =====
    // 对于只有少量章节的EPUB，很多标准规则可能不适用
    if total_spine <= 10 {
        // 如果有一定的文本内容，倾向于认为是正文章节
        if text_length > 200 {
            println!("  基于小型EPUB特殊处理分类为: ChapterBody (文本长度>200)");
            return ChapterType::ChapterBody;
        }
        // 即使文本较少，但如果标题看起来像章节
        let title_lower = title.to_lowercase();
        if title_lower.contains("第") || title_lower.contains("chapter") || title_lower.contains("节") || title_lower.contains("回") {
            println!("  基于小型EPUB特殊处理分类为: ChapterBody (标题包含章节关键词)");
            return ChapterType::ChapterBody;
        }
        // 如果spine_id包含数字
        if spine_id.chars().any(|c| c.is_ascii_digit()) {
            println!("  基于小型EPUB特殊处理分类为: ChapterBody (spine_id包含数字)");
            return ChapterType::ChapterBody;
        }
    }

    println!("  无法分类，返回: Unknown");
    ChapterType::Unknown
}
use crate::db::dbService;
use tauri::AppHandle;
use tauri::Emitter;
use tauri::Manager;
use std::io::{Write, Read};

/// 分片后的章节内容块
#[derive(serde::Serialize, tauri_ts_generator::TS)]
#[ts(export)]
pub struct ContentChunk {
    pub chapter_index: i32,
    pub chapter_title: String,
    pub chunk_index: usize,
    pub total_chunks: usize,
    pub content: String,
}

/// 处理后的章节数据（含分片和系统提示词）
#[derive(serde::Serialize, tauri_ts_generator::TS)]
#[ts(export)]
pub struct ProcessedChaptersResult {
    pub system_prompt: String,
    pub chunks: Vec<ContentChunk>,
    pub total_chapters: usize,
    pub total_chunks: usize,
}

// ============================================================
// LLM 响应解析
// ============================================================

/// LLM 返回的单个场景
#[derive(serde::Deserialize, Debug)]
struct LlmSceneItem {
    #[serde(rename = "type")]
    scene_type: String,
    content: String,
    character: Option<String>,
    emotion: Option<String>,
    speed: Option<f64>,
    pitch: Option<f64>,
    pause_duration: Option<f64>,
}

/// LLM 返回的完整响应
#[derive(serde::Deserialize, Debug)]
struct LlmResponse {
    scenes: Vec<LlmSceneItem>,
}

// ============================================================
// 章节角色分析
// ============================================================

/// LLM 返回的章节角色信息
#[derive(serde::Deserialize, Debug)]
pub struct ChapterCharacterInfo {
    pub name: String,
    pub traits: String,
    pub description: String,
}

/// 章节角色分析结果
#[derive(serde::Deserialize, Debug)]
pub struct ChapterCharacterAnalysis {
    pub characters: Vec<ChapterCharacterInfo>,
    pub main_perspective: String,
}

// ============================================================
// TTS 生成进度事件
// ============================================================

/// 进度事件载荷，通过 Tauri event 发送到前端
#[derive(serde::Serialize, Clone)]
pub struct TtsGenerationProgress {
    pub phase: String,       // "start" | "processing" | "complete" | "error"
    pub total_chunks: usize,
    pub processed_chunks: usize,
    pub message: String,
}

/// 默认分片大小（字符数），按中文字符估算
const CHUNK_SIZE: usize = 2000;

/// DeepSeek API 配置（先写死，后续可改为配置化）
const DEEPSEEK_API_URL: &str = "https://api.deepseek.com/chat/completions";
const DEEPSEEK_MODEL: &str = "deepseek-v4-flash";

// ============================================================
// 系统提示词
// ============================================================

/// 构建系统提示词，确保 LLM 输出结构一致
///
/// 如果提供了章节角色分析结果，会将其融入 prompt 中，约束 character 字段只能从分析出的角色中选择。
fn build_system_prompt(character_analysis: Option<&ChapterCharacterAnalysis>) -> String {
    let base = r#"你是一个专业的文本转语音（TTS）剧本生成助手。你的任务是将小说章节内容转换为结构化的 TTS 剧本输出。

## 输出格式要求
你必须严格按照以下 JSON 结构输出，不要包含任何额外的解释或标记：

```json
{
  "scenes": [
    {
      "type": "dialogue" | "thought",
      "content": "文本内容",
      "character": "角色名称（必填）",
      "emotion": "neutral | happy | sad | angry | surprised | fearful | calm | excited",
      "speed": 1.0,
      "pitch": 1.0,
      "pause_duration": 0.5
    }
  ]
}
```

## 字段说明
- **type**: 内容类型
  - `dialogue`: 角色对话
  - `thought`: 角色心理活动
- **content**: 实际的文本内容，保留原文风格
- **character**: 角色名称，**type 为 dialogue 或 thought 时必填**
  - `dialogue`: 说话的角色名称
  - `thought`: 该角色是谁的内心独白，必须填写角色名称，不能为空
  - 不要出现"我"、"你"这类人称代词，直接写角色名如"张三"、"李四"
  - 不要出现"张三同学","李四先生"这类带称谓的角色名，直接写"张三"、"李四"
- **emotion**: 情感基调，根据上下文判断
  - `neutral`: 中性/平静叙述
  - `happy`: 高兴/愉悦
  - `sad`: 悲伤/失落
  - `angry`: 愤怒/不满
  - `surprised`: 惊讶/意外
  - `fearful`: 恐惧/紧张
  - `calm`: 平和/舒缓
  - `excited`: 兴奋/热烈
- **speed**: 语速倍率，范围 0.5~2.0，默认 1.0
- **pitch**: 音调倍率，范围 0.5~2.0，默认 1.0
- **pause_duration**: 该句前的停顿时间（秒），用于语音合成时拼接整体的节奏控制
  - 段落/场景切换: 0.8~1.5
  - 正常对话切换: 0.3~0.5
  - 紧张急促: 0.1~0.3
  - 重要叙述/高潮前: 0.5~0.8
  - 默认 0.5

## 处理规则
1. 保留原文所有的对话内容，不要删减或改写
2. 正确识别说话角色，将对话归属到对应角色
3. 旁白部分保持原文的文学性和描写细节
4. 根据上下文内容判断合适的情感标签
5. 每个 scenes 条目应该是一个完整的语义单元（一句完整的话或一个完整的段落）
6. 多个角色连续对话时，每个角色的每句话单独成一个条目

## 重要提示
- 只输出 JSON，不要包含任何其他文字
- 确保 JSON 格式正确、可解析
- 如果原文包含引号，在 JSON 中正确转义"#;

    let mut result = base.to_string();

    if let Some(analysis) = character_analysis {
        if !analysis.characters.is_empty() {
            let character_names: Vec<&str> = analysis.characters.iter().map(|c| c.name.as_str()).collect();

            result.push_str("\n\n## 本章节角色信息\n");
            result.push_str(&format!("【主视角角色】{}\n\n", analysis.main_perspective));
            result.push_str("【本章节角色列表】\n");
            for ch in &analysis.characters {
                result.push_str(&format!("- {}: {}\n", ch.name, ch.traits));
            }
            result.push_str("\n## 角色约束（重要）\n");
            result.push_str("character 字段必须严格使用上述角色列表中的标准名称，");
            result.push_str(&format!("只能从以下角色中选择：{}。", character_names.join("、")));
            result.push_str("不要使用角色列表以外的任何名称，不要添加称谓（如同学、先生、小姐），不要使用人称代词。");
        }
    }

    result
}

// 章节角色分析 LLM 调用
/// 构建角色分析提示词，让 LLM 分析章节中的角色信息
fn build_character_analysis_prompt() -> String {
    r#"你是一个专业的小说角色分析助手。你的任务是分析给定的小说章节内容，提取出本章节中出现的所有角色信息。

## 输出格式要求
你必须严格按照以下 JSON 结构输出，不要包含任何额外的解释或标记：

{
  "characters": [
    {
      "name": "角色名称",
      "traits": "角色在本章节中展现出的性格特征",
      "description": "角色在本章节中的定位与表现简介"
    }
  ],
  "main_perspective": "主视角角色名称"
}

## 字段说明
- **characters**: 本章节中出现的所有角色列表
  - **name**: 角色名称，使用原文中对该角色的标准称呼
  - **traits**: 该角色在本章节中展现出的性格特征、说话风格、行为特点等
  - **description**: 该角色在本章节中的定位和表现简介
- **main_perspective**: 本章节的主视角角色名称
  - **必须**是 characters 列表中的某个角色名称
  - 主视角通常是本章节中戏份最多、故事以其视角展开的角色
  - 如果本章是旁白/第三人称视角，选出本章中心人物作为主视角

## 分析规则
1. 仔细阅读整个章节内容，识别所有出现的角色
2. 角色包括：有名有姓的角色、有明显对话和行为的角色
3. 不要遗漏主要角色
4. 对每个角色分析其在本章节中的性格特征和行为特点
5. 主视角必须从 characters 列表中选取，不能是"无"、"旁白"、"narrator"或其他不存在的值

## 重要提示
- 只输出 JSON，不要包含任何其他文字
- 确保 JSON 格式正确、可解析
- 角色名称要准确，使用原文中的称呼"#.to_string()
}


/// 对完整章节内容进行角色分析：调用 LLM 提取角色列表和主视角
fn analyze_chapter_characters(api_key: &str, chapter_content: &str) -> Result<ChapterCharacterAnalysis, String> {
    println!("[analyze_chapter_characters] 开始分析章节角色...");
    println!("[analyze_chapter_characters] 章节内容长度: {} 字符", chapter_content.chars().count());

    let system_prompt = build_character_analysis_prompt();
    let response = call_deepseek_api(api_key, &system_prompt, chapter_content)?;

    println!("[analyze_chapter_characters] LLM 响应: {}", response);

    let analysis: ChapterCharacterAnalysis = serde_json::from_str(&response).map_err(|e| {
        eprintln!("[analyze_chapter_characters] JSON 解析失败: {}, raw={}", e, &response[..response.len().min(300)]);
        format!("角色分析响应解析失败: {}", e)
    })?;

    println!("[analyze_chapter_characters] 角色分析完成");
    println!("[analyze_chapter_characters]   主视角角色: {}", analysis.main_perspective);
    println!("[analyze_chapter_characters]   共 {} 个角色:", analysis.characters.len());
    for ch in &analysis.characters {
        println!("[analyze_chapter_characters]     - {}: {}", ch.name, ch.traits);
    }

    // 验证主视角是否在角色列表中
    if !analysis.characters.is_empty() {
        let main_in_list = analysis.characters.iter().any(|c| c.name == analysis.main_perspective);
        if !main_in_list {
            eprintln!("[analyze_chapter_characters] 警告: 主视角 '{}' 不在角色列表中", analysis.main_perspective);
        }
    }

    Ok(analysis)
}

// 文本分片
/// 将文本按字符数分片，尽量在句子边界处断开
fn split_content(content: &str, max_chunk_size: usize) -> Vec<String> {
    let total_chars = content.chars().count();
    println!(
        "[split_content] 开始分片: 总字符数={}, 最大分片大小={}",
        total_chars, max_chunk_size
    );

    if total_chars <= max_chunk_size {
        println!("[split_content] 内容未超过分片大小，无需分片");
        return vec![content.to_string()];
    }

    let mut chunks = Vec::new();
    let mut start = 0;
    let chars: Vec<char> = content.chars().collect();
    let total_len = chars.len();

    while start < total_len {
        let end = if start + max_chunk_size >= total_len {
            total_len
        } else {
            let search_end = (start + max_chunk_size).min(total_len);
            let search_start = (search_end.saturating_sub(200)).max(start);
            let mut split_pos = search_end;

            for i in (search_start..search_end).rev() {
                if matches!(chars[i], '。' | '！' | '？' | '\n' | '；' | '”' | '…') {
                    split_pos = i + 1;
                    break;
                }
            }

            if split_pos == start {
                // 没有找到合适的断句点，强制在 max_chunk_size 处断开
                split_pos = search_end;
                println!(
                    "[split_content] 未找到断句点，强制在位置 {} 处断开",
                    split_pos
                );
            }

            split_pos
        };

        let chunk: String = chars[start..end].iter().collect();
        let chunk_chars = chunk.chars().count();
        println!(
            "[split_content] 分片 {}: 位置 {}..{}, 字符数={}",
            chunks.len(),
            start,
            end,
            chunk_chars
        );
        chunks.push(chunk);
        start = end;
    }

    println!(
        "[split_content] 分片完成: 共 {} 片",
        chunks.len()
    );
    chunks
}

// 章节处理
/// 处理选中的章节：从数据库提取内容、分片、生成系统提示词
pub fn process_selected_chapters(
    app: &AppHandle,
    chapter_ids: Vec<i32>,
    character_analysis: Option<&ChapterCharacterAnalysis>,
) -> Result<ProcessedChaptersResult, String> {
    println!(
        "============================================================"
    );
    println!(
        "[process_selected_chapters] 开始处理章节, 请求章节 IDs: {:?}",
        chapter_ids
    );
    println!(
        "============================================================"
    );

    // 1. 从数据库获取章节内容
    println!(
        "[process_selected_chapters] 正在从数据库查询章节内容..."
    );
    let chapters = dbService::get_chapters_content_by_ids(app, &chapter_ids)
        .map_err(|e| {
            eprintln!("[process_selected_chapters] 数据库查询失败: {}", e);
            format!("获取章节内容失败: {}", e)
        })?;

    println!(
        "[process_selected_chapters] 数据库查询成功, 获取到 {} 个章节",
        chapters.len()
    );

    if chapters.is_empty() {
        eprintln!("[process_selected_chapters] 未找到指定章节");
        return Err("未找到指定章节".to_string());
    }

    // 打印每个章节的基本信息
    for (idx, (chapter_index, title, content)) in chapters.iter().enumerate() {
        let content_len = content.chars().count();
        println!(
            "[process_selected_chapters]   章节 {}: index={}, title='{}', 字符数={}",
            idx + 1,
            chapter_index,
            title,
            content_len
        );
    }

    // 2. 生成系统提示词
    println!("[process_selected_chapters] 正在生成系统提示词...");
    let system_prompt = build_system_prompt(character_analysis);
    println!(
        "[process_selected_chapters] 系统提示词生成完成, 长度={} 字符",
        system_prompt.chars().count()
    );

    // 3. 分片处理
    println!("[process_selected_chapters] 开始对章节内容进行分片...");
    let mut chunks = Vec::new();
    let mut total_chunk_count = 0;

    for (chapter_index, title, content) in &chapters {
        let content_chunks = split_content(content, CHUNK_SIZE);
        let total = content_chunks.len();
        println!(
            "[process_selected_chapters]   章节 '{}' (index={}) 被分为 {} 片",
            title, chapter_index, total
        );

        for (i, chunk_content) in content_chunks.iter().enumerate() {
            chunks.push(ContentChunk {
                chapter_index: *chapter_index,
                chapter_title: title.clone(),
                chunk_index: i,
                total_chunks: total,
                content: chunk_content.clone(),
            });
            total_chunk_count += 1;
        }
    }

    println!(
        "[process_selected_chapters] 分片完成: {} 个章节 -> {} 个分片",
        chapters.len(),
        total_chunk_count
    );

    let result = ProcessedChaptersResult {
        system_prompt,
        total_chapters: chapters.len(),
        total_chunks: total_chunk_count,
        chunks,
    };

    println!(
        "[process_selected_chapters] 处理完成, 返回结果: total_chapters={}, total_chunks={}",
        result.total_chapters, result.total_chunks
    );
    println!(
        "============================================================"
    );

    Ok(result)
}

// 大模型 API 调用
#[derive(serde::Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<Message>,
    #[serde(rename = "response_format")]
    response_format: ResponseFormat,
}

#[derive(serde::Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(serde::Serialize)]
struct ResponseFormat {
    #[serde(rename = "type")]
    type_: String,
}

#[derive(serde::Deserialize)]
struct DeepSeekResponse {
    choices: Vec<Choice>,
}

#[derive(serde::Deserialize)]
struct Choice {
    message: ResponseMessage,
}

#[derive(serde::Deserialize)]
struct ResponseMessage {
    content: String,
}

/// 调用 DeepSeek API，发送系统提示词 + 用户内容，返回 LLM 响应文本
pub fn call_deepseek_api(
    api_key: &str,
    system_prompt: &str,
    user_content: &str,
) -> Result<String, String> {
    println!("[call_deepseek_api] 开始调用 DeepSeek API");
    println!("[call_deepseek_api] API URL: {}", DEEPSEEK_API_URL);
    println!("[call_deepseek_api] Model: {}", DEEPSEEK_MODEL);
    println!(
        "[call_deepseek_api] system_prompt 长度: {} 字符",
        system_prompt.chars().count()
    );
    println!(
        "[call_deepseek_api] user_content 长度: {} 字符",
        user_content.chars().count()
    );
    println!(
        "[call_deepseek_api] api_key 前8位: {}...",
        if api_key.len() > 8 {
            &api_key[..8]
        } else {
            "***"
        }
    );

    let request_body = ChatCompletionRequest {
        model: DEEPSEEK_MODEL.to_string(),
        messages: vec![
            Message {
                role: "system".to_string(),
                content: system_prompt.to_string(),
            },
            Message {
                role: "user".to_string(),
                content: user_content.to_string(),
            },
        ],
        response_format: ResponseFormat {
            type_: "json_object".to_string(),
        },
    };

    let json_body = serde_json::to_string(&request_body).map_err(|e| {
        eprintln!("[call_deepseek_api] JSON 序列化失败: {}", e);
        format!("JSON 序列化失败: {}", e)
    })?;

    println!("[call_deepseek_api] 请求体大小: {} 字节", json_body.len());

    // 发送 HTTP 请求
    println!("[call_deepseek_api] 正在发送请求...");
    let response = ureq::post(DEEPSEEK_API_URL)
        .set("Content-Type", "application/json")
        .set("Authorization", &format!("Bearer {}", api_key))
        .send_string(&json_body)
        .map_err(|e| {
            eprintln!("[call_deepseek_api] HTTP 请求失败: {}", e);
            format!("API 请求失败: {}", e)
        })?;

    let status = response.status();
    println!("[call_deepseek_api] 响应状态码: {}", status);

    let response_text = response.into_string().map_err(|e| {
        eprintln!("[call_deepseek_api] 读取响应体失败: {}", e);
        format!("读取响应失败: {}", e)
    })?;

    if status != 200 {
        eprintln!(
            "[call_deepseek_api] API 返回错误: status={}, body={}",
            status, response_text
        );
        return Err(format!("API 返回错误 ({}): {}", status, response_text));
    }

    println!(
        "[call_deepseek_api] 响应体大小: {} 字节",
        response_text.len()
    );

    // 解析响应
    let deepseek_response: DeepSeekResponse = serde_json::from_str(&response_text).map_err(|e| {
        eprintln!(
            "[call_deepseek_api] 响应 JSON 解析失败: {}, raw={}",
            e, &response_text[..response_text.len().min(200)]
        );
        format!("响应解析失败: {}", e)
    })?;

    if let Some(choice) = deepseek_response.choices.first() {
        let content = &choice.message.content;
        println!(
            "[call_deepseek_api] API 调用成功, 响应内容长度: {} 字符",
            content.chars().count()
        );
        println!(
            "[call_deepseek_api] 响应内容前 400 字: {}",
            &content.chars().take(400).collect::<String>()
        );
        Ok(content.clone())
    } else {
        eprintln!("[call_deepseek_api] 响应中没有 choices");
        Err("API 响应中没有 choices".to_string())
    }
}

/// 从数据库获取活跃的 API key，并调用 DeepSeek API
pub fn call_llm_with_active_key(
    app: &AppHandle,
    system_prompt: &str,
    user_content: &str,
) -> Result<String, String> {
    println!("[call_llm_with_active_key] 从数据库获取活跃 API key...");

    let api_key_data = dbService::get_active_api_key(app).map_err(|e| {
        eprintln!("[call_llm_with_active_key] 查询 API key 失败: {}", e);
        format!("查询 API key 失败: {}", e)
    })?;

    let api_key = match api_key_data {
        Some(data) => {
            println!(
                "[call_llm_with_active_key] 找到活跃 API key: provider='{}', model='{:?}'",
                data.provider, data.model
            );
            data.api_key
        }
        None => {
            eprintln!("[call_llm_with_active_key] 未设置活跃的 API key");
            return Err("请先在设置中添加并激活大模型 API 密钥".to_string());
        }
    };

    call_deepseek_api(&api_key, system_prompt, user_content)
}

// ============================================================
// Tauri 命令
// ============================================================

/// Tauri 命令：处理选中的章节（提取+分片+系统提示词）
#[tauri::command]
pub fn process_chapters(
    app: AppHandle,
    chapter_ids: Vec<i32>,
) -> Result<ProcessedChaptersResult, String> {
    process_selected_chapters(&app, chapter_ids, None)
}

/// Tauri 命令：对单个内容分片调用大模型 API
#[tauri::command]
pub fn call_llm_on_chunk(
    app: AppHandle,
    system_prompt: String,
    content: String,
) -> Result<String, String> {
    println!("[call_llm_on_chunk] 收到调用请求");
    println!(
        "[call_llm_on_chunk] system_prompt 长度={}, content 长度={}",
        system_prompt.chars().count(),
        content.chars().count()
    );
    call_llm_with_active_key(&app, &system_prompt, &content)
}

// ============================================================
// 后台 TTS 生成（不阻塞主进程）
// ============================================================

/// 后台运行 TTS 生成：角色分析 → 分片 → 调 API → 解析 → 存库
fn run_tts_generation(app: &AppHandle, novel_id: i32, chapter_indices: &[i32]) -> Result<(), String> {
    println!("============================================================");
    println!("[run_tts_generation] 开始后台 TTS 生成, novel_id={}, 章节索引: {:?}", novel_id, chapter_indices);
    println!("============================================================");

    // 1. 获取 API key
    println!("[run_tts_generation] 使用硬编码 API key");
    let api_key = "sk-dad3d16503da4cab83e3916336c9a52a";

    // 2. 获取章节完整信息（通过 novel_id + chapter_index 定位）
    println!("[run_tts_generation] 查询章节信息...");
    let chapters = dbService::get_chapters_by_novel_and_indices(app, novel_id as i64, chapter_indices).map_err(|e| {
        eprintln!("[run_tts_generation] 查询章节失败: {}", e);
        format!("查询章节失败: {}", e)
    })?;

    if chapters.is_empty() {
        return Err("未找到指定章节".to_string());
    }

    // 3. 构建基础系统提示词（不包含角色分析信息，后续每章节单独构建）
    println!("[run_tts_generation] 构建基础系统提示词...");
    let base_system_prompt = build_system_prompt(None);

    // 4. 统计总分片数
    let mut total_chunks: usize = 0;
    for (_, _, content, _, _) in &chapters {
        total_chunks += split_content(content, CHUNK_SIZE).len();
    }

    println!("[run_tts_generation] 共 {} 个章节, {} 个分片", chapters.len(), total_chunks);

    // 5. 发送开始事件
    let _ = app.emit("tts-progress", TtsGenerationProgress {
        phase: "start".to_string(),
        total_chunks,
        processed_chunks: 0,
        message: format!("开始处理 {} 个章节, 共 {} 个分片", chapters.len(), total_chunks),
    });

    // 6. 逐章节处理：角色分析 → 构建增强提示词 → 分片 → 调 LLM → 解析 → 存库
    let mut processed_chunks: usize = 0;
    let mut failed_chunks: usize = 0;

    for (chapter_index, chapter_title, content, novel_id, chapter_id) in &chapters {
        println!("============================================================");
        println!("[run_tts_generation] 开始处理章节 '{}' (index={})", chapter_title, chapter_index);
        println!("============================================================");

        // 6a. 角色分析：整章内容调用 LLM，提取角色列表和主视角
        println!("[run_tts_generation]   正在进行章节角色分析...");
        let character_analysis = match analyze_chapter_characters(api_key, content) {
            Ok(analysis) => {
                println!("[run_tts_generation]   角色分析成功: 主视角='{}', {} 个角色",
                    analysis.main_perspective, analysis.characters.len());
                analysis
            }
            Err(e) => {
                eprintln!("[run_tts_generation]   角色分析失败: {}", e);
                println!("[run_tts_generation]   角色分析失败，使用基础系统提示词继续处理");
                ChapterCharacterAnalysis {
                    characters: vec![],
                    main_perspective: String::new(),
                }
            }
        };

        // 6b. 构建系统提示词（包含角色信息，约束 character 字段）
        let system_prompt = if character_analysis.characters.is_empty() {
            base_system_prompt.clone()
        } else {
            let enriched = build_system_prompt(Some(&character_analysis));
            println!("[run_tts_generation]   增强系统提示词已构建 (长度={})", enriched.chars().count());
            enriched
        };

        // 6c. 分片处理
        let content_chunks = split_content(content, CHUNK_SIZE);
        println!(
            "[run_tts_generation]   章节 '{}' 被分为 {} 个分片",
            chapter_title, content_chunks.len()
        );

        for (chunk_idx, chunk_content) in content_chunks.iter().enumerate() {
            println!(
                "[run_tts_generation]   分片 {}/{} (章节 '{}')",
                processed_chunks + 1,
                total_chunks,
                chapter_title
            );

            // 调 LLM API
            let response = match call_deepseek_api(api_key, &system_prompt, chunk_content) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("[run_tts_generation] API 调用失败 (分片 {}): {}", processed_chunks, e);
                    failed_chunks += 1;
                    processed_chunks += 1;
                    let _ = app.emit("tts-progress", TtsGenerationProgress {
                        phase: "processing".to_string(),
                        total_chunks,
                        processed_chunks,
                        message: format!("分片 {} 失败: {}", processed_chunks, e),
                    });
                    continue;
                }
            };

            // 解析 JSON 响应
            let llm_response: LlmResponse = match serde_json::from_str(&response) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!(
                        "[run_tts_generation] 解析 LLM 响应失败: {}, raw={}",
                        e,
                        &response[..response.len().min(200)]
                    );
                    failed_chunks += 1;
                    processed_chunks += 1;
                    continue;
                }
            };

            // 插入分片记录（存原始输入/响应用于溯源，每分片一条）
            let chunk_id = match dbService::insert_tts_chunk(app, &dbService::TtsChunkData {
                id: None,
                novel_id: *novel_id,
                chapter_id: *chapter_id,
                chunk_index: chunk_idx as i32,
                system_prompt: Some(system_prompt.clone()),
                user_content: Some(chunk_content.clone()),
                raw_request: None,
                raw_response: Some(response.clone()),
            }) {
                Ok(id) => {
                    println!("[run_tts_generation]   分片记录已写入 (id={})", id);
                    Some(id)
                }
                Err(e) => {
                    eprintln!("[run_tts_generation]   写入分片记录失败: {}", e);
                    None
                }
            };

            // 构建场景记录（结构化，每场景一条）
            let scenes: Vec<dbService::TtsSceneData> = llm_response
                .scenes
                .iter()
                .enumerate()
                .map(|(i, scene)| dbService::TtsSceneData {
                    chunk_id,
                    novel_id: *novel_id,
                    chapter_id: *chapter_id,
                    chunk_index: chunk_idx as i32,
                    scene_index: i as i32,
                    scene_type: scene.scene_type.clone(),
                    content: scene.content.clone(),
                    character_name: scene.character.clone(),
                    emotion: scene.emotion.clone().unwrap_or_else(|| "neutral".to_string()),
                    speed: scene.speed.unwrap_or(1.0),
                    pitch: scene.pitch.unwrap_or(1.0),
                    pause_duration: scene.pause_duration.unwrap_or(0.5),
                })
                .collect();

            println!(
                "[run_tts_generation]   分片 {}: 解析到 {} 个场景, 正在写入数据库...",
                processed_chunks + 1,
                scenes.len()
            );

            // 写入数据库
            if let Err(e) = dbService::insert_tts_scenes_batch(app, &scenes) {
                eprintln!("[run_tts_generation] 写入数据库失败: {}", e);
                failed_chunks += 1;
            }

            processed_chunks += 1;

            // 发送进度事件
            let _ = app.emit("tts-progress", TtsGenerationProgress {
                phase: "processing".to_string(),
                total_chunks,
                processed_chunks,
                message: format!(
                    "章节「{}」分片 {}/{} 完成 ({} 个场景)",
                    chapter_title, processed_chunks, total_chunks, scenes.len()
                ),
            });
        }

        // 6d. 标记该章节的 TTS 为已生成
        println!(
            "[run_tts_generation]   标记章节 '{}' (index={}) TTS 已生成",
            chapter_title, chapter_index
        );
        if let Err(e) = dbService::update_chapter_tts_generated(app, *chapter_id, true) {
            eprintln!("[run_tts_generation]   标记 TTS 生成状态失败: {}", e);
        }
    }

    // 7. 发送完成事件
    let status = if failed_chunks > 0 { "完成（部分失败）" } else { "全部完成" };
    println!(
        "[run_tts_generation] {}: 成功 {} / {} 分片, {} 失败",
        status,
        processed_chunks - failed_chunks,
        total_chunks,
        failed_chunks
    );

    let _ = app.emit("tts-progress", TtsGenerationProgress {
        phase: "complete".to_string(),
        total_chunks,
        processed_chunks,
        message: format!(
            "{}，成功 {}/{} 分片",
            status,
            processed_chunks - failed_chunks,
            total_chunks
        ),
    });

    println!("============================================================");
    Ok(())
}

/// Tauri 命令：启动后台 TTS 生成（不阻塞前端）
#[tauri::command]
pub fn start_tts_generation(app: AppHandle, novel_id: i32, chapter_ids: Vec<i32>) -> Result<(), String> {
    println!(
        "[start_tts_generation] 收到请求, novel_id={}, 章节索引: {:?}",
        novel_id, chapter_ids
    );

    if chapter_ids.is_empty() {
        return Err("请选择至少一个章节".to_string());
    }

    // 在后台线程运行，不阻塞主进程
    std::thread::spawn(move || {
        if let Err(e) = run_tts_generation(&app, novel_id, &chapter_ids) {
            eprintln!("[start_tts_generation] 后台处理失败: {}", e);
            let _ = app.emit("tts-progress", TtsGenerationProgress {
                phase: "error".to_string(),
                total_chunks: 0,
                processed_chunks: 0,
                message: format!("处理失败: {}", e),
            });
        }
    });

    Ok(())
}

/// Tauri 命令：获取所有已生成 TTS 剧本的章节
#[tauri::command]
pub fn get_tts_generated_chapters(app: AppHandle) -> Result<Vec<dbService::TtsGeneratedChapter>, String> {
    dbService::get_tts_generated_chapters(&app).map_err(|e| e.to_string())
}

/// Tauri 命令：获取指定章节的 TTS 摘要（角色列表、场景数等）
#[tauri::command]
pub fn get_chapter_tts_summary(app: AppHandle, chapter_id: i64) -> Result<dbService::ChapterTtsSummary, String> {
    dbService::get_chapter_tts_summary(&app, chapter_id).map_err(|e| e.to_string())
}

// ============================================================
// 章节角色语音映射
// ============================================================

/// Tauri 命令：获取所有章节（含小说信息），用于 TTS 生成页面
#[tauri::command]
pub fn get_all_chapters(app: AppHandle) -> Result<Vec<dbService::ChapterWithNovel>, String> {
    dbService::get_all_chapters_grouped_by_novel(&app).map_err(|e| e.to_string())
}

/// Tauri 命令：获取指定章节的角色→语音映射
#[tauri::command]
pub fn get_chapter_character_mappings(app: AppHandle, chapter_id: i64) -> Result<Vec<dbService::ChapterCharacterVoiceMap>, String> {
    dbService::get_character_voice_mappings_for_chapter(&app, chapter_id).map_err(|e| e.to_string())
}

/// Tauri 命令：保存章节的角色→语音映射
/// mappings 格式：[[character_name, voice_id], ...]
#[tauri::command]
pub fn save_chapter_character_mappings(app: AppHandle, chapter_id: i64, mappings: Vec<Vec<String>>) -> Result<(), String> {
    let parsed: Vec<(String, i64)> = mappings
        .iter()
        .map(|m| {
            if m.len() != 2 {
                return Err("映射格式错误，需要 [character_name, voice_id]".to_string());
            }
            let voice_id: i64 = m[1].parse().map_err(|_| "voice_id 解析失败".to_string())?;
            Ok((m[0].clone(), voice_id))
        })
        .collect::<Result<Vec<_>, String>>()?;

    dbService::save_character_voice_mappings_for_chapter(&app, chapter_id, &parsed)
        .map_err(|e| e.to_string())
}

// ============================================================
// 音频生成
// ============================================================

/// 音频生成进度事件
#[derive(serde::Serialize, Clone)]
pub struct TtsAudioProgress {
    pub phase: String,       // "start" | "processing" | "complete" | "error"
    pub total: usize,
    pub processed: usize,
    pub message: String,
}

/// 后台运行音频生成
fn run_chapter_audio_generation(app: &AppHandle, chapter_id: i64) -> Result<(), String> {
    println!("[run_chapter_audio_generation] 开始音频生成，chapter_id={}", chapter_id);

    // 1. 获取章节所有 TTS 场景
    let scenes = dbService::get_tts_scenes_with_id_by_chapter(app, chapter_id)
        .map_err(|e| format!("获取 TTS 场景失败: {}", e))?;

    if scenes.is_empty() {
        return Err("该章节没有 TTS 场景".to_string());
    }

    // 2. 获取角色→语音映射
    let mappings = dbService::get_character_voice_mappings_for_chapter(app, chapter_id)
        .map_err(|e| format!("获取角色映射失败: {}", e))?;

    let voice_map: std::collections::HashMap<String, i64> = mappings
        .iter()
        .map(|m| (m.character_name.clone(), m.voice_id))
        .collect();

    // 3. 获取配置中的 mp3 输出路径
    let config = app.state::<crate::state::appState::AppState>();
    let mp3_path = config.config.lock().unwrap().mp3_path.clone();

    if mp3_path.as_os_str().is_empty() {
        return Err("请先在设置中配置音频输出目录 (mp3_path)".to_string());
    }

    // 从 scenes 获取 novel_id
    let novel_id = scenes[0].novel_id;
    let output_dir = mp3_path.join(novel_id.to_string()).join(chapter_id.to_string());
    std::fs::create_dir_all(&output_dir).map_err(|e| format!("创建输出目录失败: {}", e))?;

    let total = scenes.len();
    let mut processed = 0;
    let mut failed = 0;

    // 发送开始事件
    let _ = app.emit("tts-audio-progress", TtsAudioProgress {
        phase: "start".to_string(),
        total,
        processed: 0,
        message: format!("开始音频生成，共 {} 条场景", total),
    });

    // 4. 遍历场景生成音频
    for scene in &scenes {
        let character_name = match &scene.character_name {
            Some(name) if !name.is_empty() => name.clone(),
            _ => {
                println!("[run_chapter_audio_generation] 场景 {} 无角色名，跳过", scene.id);
                processed += 1;
                continue;
            }
        };

        // 查找角色映射
        let voice_id = match voice_map.get(&character_name) {
            Some(id) => id,
            None => {
                println!("[run_chapter_audio_generation] 角色 '{}' 未配置语音映射，跳过", character_name);
                failed += 1;
                processed += 1;
                continue;
            }
        };

        // 获取语音配置
        let voice_config = match dbService::get_character_voice_by_id(app, *voice_id) {
            Ok(v) => v,
            Err(e) => {
                println!("[run_chapter_audio_generation] 获取语音配置失败 (voice_id={}): {}", voice_id, e);
                failed += 1;
                processed += 1;
                continue;
            }
        };

        // 调用 GPT-SoVITS API 生成音频（最多重试 3 次）
        let max_retries = 3;
        let audio_bytes = {
            let mut last_err = String::new();
            let mut result = None;
            for attempt in 1..=max_retries {
                let adjusted_speed = (scene.speed - 0.2).max(0.5);
                match call_gptsovits_tts(&voice_config, &scene.content, &scene.emotion, adjusted_speed) {
                    Ok(bytes) => {
                        result = Some(bytes);
                        break;
                    }
                    Err(e) => {
                        last_err = e;
                        println!(
                            "[run_chapter_audio_generation] 音频生成失败 (scene_id={}, 第{}/{}){}: {}",
                            scene.id,
                            attempt,
                            max_retries,
                            if attempt < max_retries { "，即将重试" } else { "，已达最大重试次数" },
                            last_err,
                        );
                    }
                }
            }
            match result {
                Some(bytes) => bytes,
                None => {
                    eprintln!("[run_chapter_audio_generation] 音频生成最终失败 (scene_id={}): {}", scene.id, last_err);
                    failed += 1;
                    processed += 1;
                    continue;
                }
            }
        };

        // 写入音频文件
        let safe_name = character_name.replace(|c: char| !c.is_alphanumeric() && c != '-' && c != '_', "_");
        let audio_filename = format!("{}_{}_{}.wav", scene.scene_index, safe_name, scene.id);
        let audio_path = output_dir.join(&audio_filename);

        if let Err(e) = save_audio_file(&audio_path, &audio_bytes) {
            println!("[run_chapter_audio_generation] 保存音频文件失败: {}", e);
            failed += 1;
            processed += 1;
            continue;
        }

        // 更新数据库中的 audio_path
        let audio_path_str = audio_path.to_string_lossy().to_string();
        if let Err(e) = dbService::update_tts_script_audio_path(app, scene.id, Some(&audio_path_str)) {
            println!("[run_chapter_audio_generation] 更新音频路径失败: {}", e);
        }

        processed += 1;

        // 发送进度事件
        let _ = app.emit("tts-audio-progress", TtsAudioProgress {
            phase: "processing".to_string(),
            total,
            processed,
            message: format!("场景 {}/{} 完成 — {}", processed, total, character_name),
        });
    }

    // 5. 更新章节 audio_path 指向目录
    let dir_path_str = output_dir.to_string_lossy().to_string();
    let _ = dbService::update_chapter_audio_path(app, chapter_id, Some(&dir_path_str));

    // 6. 发送完成事件
    let status = if failed > 0 { "部分失败" } else { "全部完成" };
    let _ = app.emit("tts-audio-progress", TtsAudioProgress {
        phase: "complete".to_string(),
        total,
        processed,
        message: format!("{}，成功 {}/{} 条", status, processed - failed, total),
    });

    println!("[run_chapter_audio_generation] 完成: {}/{} 成功, {} 失败", processed - failed, total, failed);
    Ok(())
}

/// 调用 GPT-SoVITS TTS API 生成音频
fn call_gptsovits_tts(
    voice: &dbService::CharacterVoiceData,
    text: &str,
    emotion: &str,
    speed: f64,
) -> Result<Vec<u8>, String> {
    let base = voice.api_base_url.trim_end_matches('/').to_string();
    let url = format!("{}/tts", base);

    // 构建查询参数
    let params = [
        ("text", text),
        ("text_lang", &voice.text_lang),
        ("ref_audio_path", &voice.ref_audio_path),
        ("prompt_lang", &voice.prompt_lang),
        ("prompt_text", voice.prompt_text.as_deref().unwrap_or("")),
        ("emotion", emotion),
        ("speed", &speed.to_string()),
    ];

    let query_string: String = params
        .iter()
        .map(|(k, v)| format!("{}={}", urlencoding(k), urlencoding(v)))
        .collect::<Vec<_>>()
        .join("&");

    let full_url = format!("{}?{}", url, query_string);

    println!("[call_gptsovits_tts] 请求: {} (文本长度={})", base, text.len());

    let response = ureq::get(&full_url)
        .call()
        .map_err(|e| format!("TTS API 请求失败: {}", e))?;

    let status = response.status();
    if status != 200 {
        let body = response.into_string().unwrap_or_default();
        return Err(format!("TTS API 返回错误 ({}): {}", status, body));
    }

    let mut bytes: Vec<u8> = Vec::new();
    let mut reader = response.into_reader();
    reader.read_to_end(&mut bytes).map_err(|e| format!("读取音频数据失败: {}", e))?;

    if bytes.is_empty() {
        return Err("TTS API 返回空音频数据".to_string());
    }

    println!("[call_gptsovits_tts] 成功获取音频: {} 字节", bytes.len());
    Ok(bytes)
}

/// URL 编码（简单实现，仅编码中文和特殊字符）
fn urlencoding(input: &str) -> String {
    let mut result = String::new();
    for byte in input.bytes() {
        match byte {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                result.push(byte as char);
            }
            b' ' => result.push_str("%20"),
            _ => {
                result.push_str(&format!("%{:02X}", byte));
            }
        }
    }
    result
}

/// 保存音频文件到磁盘
fn save_audio_file(path: &std::path::Path, data: &[u8]) -> Result<(), String> {
    let mut file = std::fs::File::create(path).map_err(|e| format!("创建文件失败: {}", e))?;
    file.write_all(data).map_err(|e| format!("写入文件失败: {}", e))?;
    println!("[save_audio_file] 已保存: {:?} ({} 字节)", path, data.len());
    Ok(())
}

/// Tauri 命令：后台生成指定章节的音频
#[tauri::command]
pub fn generate_chapter_audio(app: AppHandle, chapter_id: i64) -> Result<(), String> {
    println!("[generate_chapter_audio] 收到请求, chapter_id={}", chapter_id);

    std::thread::spawn(move || {
        if let Err(e) = run_chapter_audio_generation(&app, chapter_id) {
            eprintln!("[generate_chapter_audio] 后台处理失败: {}", e);
            let _ = app.emit("tts-audio-progress", TtsAudioProgress {
                phase: "error".to_string(),
                total: 0,
                processed: 0,
                message: format!("音频生成失败: {}", e),
            });
        }
    });

    Ok(())
}

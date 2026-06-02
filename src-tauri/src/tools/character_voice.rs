use std::fs;
use tauri::{AppHandle, Manager};
use crate::db::db_service::{self, CharacterVoiceData};

fn generate_tts_config(app: &AppHandle, voice: &CharacterVoiceData) -> Result<String, String> {
    let config_dir = app
        .path()
        .resource_dir()
        .map_err(|e| format!("获取资源目录失败: {}", e))?
        .join("config")
        .join("tts_infer");

    fs::create_dir_all(&config_dir).map_err(|e| format!("创建配置目录失败: {}", e))?;

    let file_path = config_dir.join(format!("{}.yaml", voice.character_name));

    let mut yaml = String::from("custom:\n");
    yaml.push_str("  bert_base_path: GPT_SoVITS/pretrained_models/chinese-roberta-wwm-ext-large\n");
    yaml.push_str("  cnhuhbert_base_path: GPT_SoVITS/pretrained_models/chinese-hubert-base\n");
    yaml.push_str("  device: cuda\n");
    yaml.push_str("  is_half: true\n");

    if let Some(ref gpt) = voice.gpt_model {
        if !gpt.is_empty() {
            yaml.push_str(&format!("  t2s_weights_path: '{}'\n", gpt));
        }
    }

    yaml.push_str("  version: v2Pro\n");

    if let Some(ref sovits) = voice.sovits_model {
        if !sovits.is_empty() {
            yaml.push_str(&format!("  vits_weights_path: '{}'\n", sovits));
        }
    }

    fs::write(&file_path, &yaml).map_err(|e| format!("写入配置文件失败: {}", e))?;

    Ok(file_path.to_string_lossy().to_string())
}

#[tauri::command]
pub fn get_all_character_voices(
    app: AppHandle,
) -> Result<Vec<CharacterVoiceData>, String> {
    db_service::get_all_character_voices(&app).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_character_voice(
    app: AppHandle,
    data: CharacterVoiceData,
) -> Result<i64, String> {
    let config_path = generate_tts_config(&app, &data)?;
    let mut data = data;
    data.config_path = Some(config_path);
    db_service::insert_character_voice(&app, &data).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_character_voice(
    app: AppHandle,
    id: i64,
    data: CharacterVoiceData,
) -> Result<(), String> {
    let config_path = generate_tts_config(&app, &data)?;
    let mut data = data;
    data.config_path = Some(config_path);
    db_service::update_character_voice(&app, id, &data).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_character_voice(
    app: AppHandle,
    id: i64,
) -> Result<(), String> {
    // 先查角色名，用于删除对应的 yaml 配置文件
    let voice = db_service::get_character_voice_by_id(&app, id).map_err(|e| e.to_string())?;

    let config_dir = app
        .path()
        .resource_dir()
        .map_err(|e| format!("获取资源目录失败: {}", e))?
        .join("config")
        .join("tts_infer");
    let yaml_path = config_dir.join(format!("{}.yaml", voice.character_name));
    let _ = fs::remove_file(&yaml_path);

    db_service::delete_character_voice(&app, id).map_err(|e| e.to_string())
}

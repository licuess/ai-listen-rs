use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use reqwest::blocking::{Client, multipart};
use serde_json::{Value, json};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const API_BASE: &str = "https://api.openai.com/v1";
const TRANSCRIBE_MODEL: &str = "gpt-4o-transcribe";
const SUMMARY_MODEL: &str = "gpt-5.5";
const VISION_MODEL: &str = "gpt-4o";

pub fn transcribe_audio(path: &str) -> Result<String, String> {
    let api_key = api_key()?;
    // 先转为 MP3 确保格式兼容，OpenAI API 对 WAV 头完整性要求严格
    let mp3_path = convert_to_mp3(path)?;
    let part = multipart::Part::file(&mp3_path)
        .map_err(|error| format!("failed to read audio file for upload: {error}"))?;
    let form = multipart::Form::new()
        .part("file", part)
        .text("model", env_or("OPENAI_TRANSCRIBE_MODEL", TRANSCRIBE_MODEL))
        .text("response_format", "text")
        .text("language", "zh");

    let response = Client::new()
        .post(format!("{API_BASE}/audio/transcriptions"))
        .bearer_auth(api_key)
        .multipart(form)
        .send()
        .map_err(|error| {
            let _ = fs::remove_file(&mp3_path);
            format!("transcription request failed: {error}")
        })?;

    let _ = fs::remove_file(&mp3_path);

    if response.status().is_success() {
        response
            .text()
            .map(|text| text.trim().to_string())
            .map_err(|error| error.to_string())
    } else {
        Err(format!(
            "transcription failed with status {}: {}",
            response.status(),
            response.text().unwrap_or_default()
        ))
    }
}

fn convert_to_mp3(input: &str) -> Result<String, String> {
    let output = std::env::temp_dir().join(format!("ai-listen-tmp-{}.mp3", timestamp()));
    let status = Command::new("ffmpeg")
        .arg("-y")
        .arg("-i")
        .arg(input)
        .args(["-acodec", "libmp3lame", "-ar", "16000", "-ac", "1"])
        .arg(&output)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|error| format!("failed to convert audio to mp3: {error}"))?;

    if !status.success() {
        return Err("audio conversion to MP3 failed".to_string());
    }

    if !output.exists() || fs::metadata(&output).map(|m| m.len()).unwrap_or(0) == 0 {
        return Err("audio conversion produced empty file".to_string());
    }

    Ok(output.to_string_lossy().to_string())
}

pub fn transcribe_audio_in_chunks(
    path: &str,
    mut on_chunk: impl FnMut(usize, usize, &str),
) -> Result<String, String> {
    let chunks = split_audio(path).unwrap_or_else(|_| vec![PathBuf::from(path)]);
    let total = chunks.len().max(1);
    let mut transcript = String::new();

    for (index, chunk) in chunks.iter().enumerate() {
        let chunk_text = transcribe_audio(&chunk.to_string_lossy())?;
        let trimmed = chunk_text.trim().to_string();
        if !transcript.is_empty() {
            transcript.push_str("\n\n");
        }
        transcript.push_str(&trimmed);
        on_chunk(index + 1, total, &trimmed);
    }

    Ok(transcript)
}

pub fn summarize_notes(notes: &str) -> Result<String, String> {
    let api_key = api_key()?;
    let model = env_or("OPENAI_SUMMARY_MODEL", SUMMARY_MODEL);
    let prompt = format!(
        "请把以下会议听记整理成中文会议纪要，包含：摘要、关键决定、待办事项、风险或未决问题。\n\n{notes}"
    );

    let body = json!({
        "model": model,
        "input": prompt
    });

    let response = Client::new()
        .post(format!("{API_BASE}/responses"))
        .bearer_auth(api_key)
        .json(&body)
        .send()
        .map_err(|error| format!("summary request failed: {error}"))?;

    if !response.status().is_success() {
        return Err(format!(
            "summary failed with status {}: {}",
            response.status(),
            response.text().unwrap_or_default()
        ));
    }

    let value = response
        .json::<Value>()
        .map_err(|error| format!("failed to parse summary response: {error}"))?;

    // 记录 token 用量
    if let Some((prompt, completion)) = crate::tokens::extract_usage_from_response(&value) {
        crate::tokens::record_usage("summarize", prompt, completion);
    }

    extract_output_text(&value).ok_or_else(|| "summary response did not include text".to_string())
}

/// 用 OpenAI 视觉模型分析截图内容，提取文字和关键信息
pub fn analyze_screenshot(image_path: &str) -> Result<String, String> {
    let api_key = api_key()?;
    let model = env_or("OPENAI_VISION_MODEL", VISION_MODEL);

    // 先用 ffmpeg 缩放截图（长边 <= 2048px），减少 base64 体积
    let resized = resize_image(image_path)?;
    let image_data = fs::read(&resized)
        .map_err(|error| format!("failed to read screenshot: {error}"))?;
    let base64_data = BASE64.encode(&image_data);

    // 如果不是临时文件则不删除
    if resized.to_string_lossy() != image_path {
        let _ = fs::remove_file(&resized);
    }

    let body = json!({
        "model": model,
        "messages": [{
            "role": "user",
            "content": [
                {
                    "type": "text",
                    "text": "请分析这张截图，提取其中的文字内容、UI元素、关键信息。用中文回答，格式清晰。"
                },
                {
                    "type": "image_url",
                    "image_url": {
                        "url": format!("data:image/png;base64,{base64_data}")
                    }
                }
            ]
        }],
        "max_tokens": 2000
    });

    let response = Client::new()
        .post(format!("{API_BASE}/chat/completions"))
        .bearer_auth(api_key)
        .json(&body)
        .send()
        .map_err(|error| format!("screenshot analysis request failed: {error}"))?;

    if !response.status().is_success() {
        return Err(format!(
            "screenshot analysis failed with status {}: {}",
            response.status(),
            response.text().unwrap_or_default()
        ));
    }

    let value = response
        .json::<Value>()
        .map_err(|error| format!("failed to parse vision response: {error}"))?;

    // 记录 token 用量
    if let Some((prompt, completion)) = crate::tokens::extract_usage_from_response(&value) {
        crate::tokens::record_usage("screenshot_analysis", prompt, completion);
    }

    value
        .pointer("/choices/0/message/content")
        .and_then(Value::as_str)
        .map(|s| s.trim().to_string())
        .ok_or_else(|| "vision response did not include content".to_string())
}

/// 用 ffmpeg 缩放图片，长边不超过 2048px。ffmpeg 不可用时返回原图
fn resize_image(input: &str) -> Result<PathBuf, String> {
    let output = std::env::temp_dir().join(format!("ai-listen-resize-{}.png", timestamp()));
    let status = match Command::new("ffmpeg")
        .arg("-y")
        .arg("-i")
        .arg(input)
        .args(["-vf", "scale='min(2048,iw)':min(2048,ih)'"])
        .arg(&output)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
    {
        Ok(status) => status,
        Err(_) => return Ok(PathBuf::from(input)), // ffmpeg 不可用，用原图
    };

    if status.success() && output.exists() {
        Ok(output)
    } else {
        Ok(PathBuf::from(input))
    }
}

fn split_audio(path: &str) -> Result<Vec<PathBuf>, String> {
    let input = Path::new(path);
    let chunk_dir = std::env::temp_dir().join(format!("ai-listen-chunks-{}", timestamp()));
    fs::create_dir_all(&chunk_dir).map_err(|error| error.to_string())?;
    let pattern = chunk_dir.join("chunk-%03d.mp3");

    let status = Command::new("ffmpeg")
        .arg("-y")
        .arg("-i")
        .arg(input)
        .args([
            "-f",
            "segment",
            "-segment_time",
            "300",
            "-reset_timestamps",
            "1",
            "-acodec",
            "libmp3lame",
            "-ar",
            "16000",
            "-ac",
            "1",
        ])
        .arg(&pattern)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|error| format!("failed to split audio for streaming transcription: {error}"))?;

    if !status.success() {
        return Err(format!("audio split exited with status {status}"));
    }

    let mut chunks = fs::read_dir(&chunk_dir)
        .map_err(|error| error.to_string())?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("mp3"))
        .collect::<Vec<_>>();
    chunks.sort();

    if chunks.is_empty() {
        Err("audio split produced no chunks".to_string())
    } else {
        Ok(chunks)
    }
}

fn api_key() -> Result<String, String> {
    // 优先从设置文件读取，其次从环境变量读取
    let settings_key = crate::settings::load_settings().openai_api_key;
    if !settings_key.is_empty() {
        return Ok(settings_key.trim().to_string());
    }
    std::env::var("OPENAI_API_KEY")
        .map(|value| value.trim().to_string())
        .map_err(|_| "OPENAI_API_KEY is not set. Please configure it in Settings.".to_string())
        .and_then(|value| {
            if value.is_empty() {
                Err("OPENAI_API_KEY is empty. Please configure it in Settings.".to_string())
            } else {
                Ok(value)
            }
        })
}

fn env_or(name: &str, fallback: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| fallback.to_string())
}

fn extract_output_text(value: &Value) -> Option<String> {
    if let Some(text) = value.get("output_text").and_then(Value::as_str) {
        return Some(text.to_string());
    }

    let output = value.get("output")?.as_array()?;
    let mut chunks = Vec::new();
    for item in output {
        let Some(content) = item.get("content").and_then(Value::as_array) else {
            continue;
        };

        for part in content {
            if let Some(text) = part.get("text").and_then(Value::as_str) {
                chunks.push(text.to_string());
            }
        }
    }

    if chunks.is_empty() {
        None
    } else {
        Some(chunks.join("\n"))
    }
}

fn timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0);
    millis.to_string()
}

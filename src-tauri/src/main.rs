mod ai;
mod audio;
mod capture;
mod email;
mod export;
mod index;
mod oauth;
mod sessions;
mod settings;
mod sms;
mod tasks;
mod tokens;
mod users;

use std::path::PathBuf;
use std::thread;

use audio::{
    AudioDevice, AudioTestResult, list_audio_devices, start_audio_recording, stop_audio_recording,
    test_audio_input,
};
use capture::{capture_screenshot, start_recording, stop_recording};
use index::{IndexStats, SearchHit};
use sessions::{SessionDetails, SessionStore};
use settings::AppSettings;
use tasks::TaskStatus;
use tokens::TokenReport;
use users::AuthResult;
use oauth::OAuthResult;

fn store() -> SessionStore {
    // 数据目录放在 src-tauri 外部，避免 Tauri dev 文件监听器误触发重建
    SessionStore::new(PathBuf::from("../ai-listen-data"))
}

#[tauri::command]
fn list_sessions() -> Result<Vec<SessionDetails>, String> {
    store().list_sessions()
}

#[tauri::command]
fn create_session(title: String) -> Result<SessionDetails, String> {
    let session = store().create_session(&title)?;
    refresh_index();
    Ok(session)
}

#[tauri::command]
fn read_session(slug: String) -> Result<SessionDetails, String> {
    store().read_session(&slug)
}

#[tauri::command]
fn save_note(slug: String, content: String) -> Result<SessionDetails, String> {
    let session = store().save_note(&slug, &content)?;
    refresh_index();
    Ok(session)
}

#[tauri::command]
fn capture_session_screenshot(slug: String) -> Result<SessionDetails, String> {
    let session_dir = store().session_dir_by_slug(&slug)?;
    let output = session_dir.join(format!("screenshot-{}.png", timestamp()));
    capture_screenshot(&output)?;
    store().read_session(&slug)
}

#[tauri::command]
fn analyze_session_screenshot(slug: String) -> Result<TaskStatus, String> {
    let session = store().read_session(&slug)?;
    let screenshot = session
        .screenshots
        .last()
        .ok_or("no screenshot found for this session")?
        .to_string();
    let task = tasks::create("screenshot_analysis", Some(slug.clone()), "等待识别");
    let task_id = task.id.clone();

    thread::spawn(move || {
        tasks::update_progress(
            &task_id,
            "running",
            "正在分析截图内容",
            Some(30),
            None,
        );
        match ai::analyze_screenshot(&screenshot) {
            Ok(analysis) => {
                tasks::update_progress(
                    &task_id,
                    "running",
                    "正在写入识别结果",
                    Some(80),
                    None,
                );
                let result = format!("\n\n## 截图识别结果\n\n{}", analysis.trim());
                match store().append_note(&slug, &result) {
                    Ok(_) => {
                        refresh_index();
                        tasks::update_progress(&task_id, "done", "截图识别完成", Some(100), None);
                    }
                    Err(error) => tasks::update_progress(&task_id, "failed", &error, Some(100), None),
                }
            }
            Err(error) => tasks::update_progress(&task_id, "failed", &error, Some(100), None),
        }
    });

    Ok(task)
}

#[tauri::command]
fn start_session_recording(slug: String) -> Result<SessionDetails, String> {
    let session_dir = store().session_dir_by_slug(&slug)?;
    let video = session_dir.join(format!("recording-{}.mp4", timestamp()));
    let pid_file = session_dir.join(".recording.pid");
    start_recording(&video, &pid_file)?;
    store().read_session(&slug)
}

#[tauri::command]
fn stop_session_recording(slug: String) -> Result<SessionDetails, String> {
    let session_dir = store().session_dir_by_slug(&slug)?;
    let pid_file = session_dir.join(".recording.pid");
    stop_recording(&pid_file)?;
    store().read_session(&slug)
}

#[tauri::command]
fn list_input_devices() -> Result<Vec<AudioDevice>, String> {
    list_audio_devices()
}

#[tauri::command]
fn test_input_device(device_id: Option<String>) -> Result<AudioTestResult, String> {
    test_audio_input(device_id.as_deref())
}

#[tauri::command]
fn start_session_audio(slug: String, device_id: Option<String>) -> Result<SessionDetails, String> {
    let session_dir = store().session_dir_by_slug(&slug)?;
    let audio = session_dir.join(format!("audio-{}.wav", timestamp()));
    let pid_file = session_dir.join(".audio.pid");
    start_audio_recording(&audio, &pid_file, device_id.as_deref())?;
    store().read_session(&slug)
}

#[tauri::command]
fn stop_session_audio(slug: String) -> Result<SessionDetails, String> {
    let session_dir = store().session_dir_by_slug(&slug)?;
    let pid_file = session_dir.join(".audio.pid");
    stop_audio_recording(&pid_file)?;
    store().read_session(&slug)
}

#[tauri::command]
fn enqueue_transcribe_latest_audio(slug: String) -> Result<TaskStatus, String> {
    let session = store().read_session(&slug)?;
    let audio = session
        .audio
        .last()
        .ok_or("no audio recording found for this session")?
        .to_string();
    let task = tasks::create("transcription", Some(slug.clone()), "等待转写");
    let task_id = task.id.clone();

    thread::spawn(move || {
        tasks::update_progress(
            &task_id,
            "running",
            "正在上传音频",
            Some(15),
            Some("准备上传音频文件..."),
        );
        tasks::update_progress(
            &task_id,
            "running",
            "正在转写音频",
            Some(55),
            Some("音频已提交，等待转写结果..."),
        );
        match ai::transcribe_audio_in_chunks(&audio, |chunk, total, chunk_text| {
            let progress = 15 + ((chunk as f32 / total as f32) * 70.0).round() as u8;
            tasks::update_progress(
                &task_id,
                "running",
                &format!("正在转写音频 {chunk}/{total}"),
                Some(progress),
                None,
            );
            tasks::append_partial(&task_id, chunk_text);
        })
        .and_then(|transcript| {
            let preview = transcript_preview(&transcript);
            tasks::update_progress(
                &task_id,
                "running",
                "正在写入转写结果",
                Some(90),
                Some(&preview),
            );
            store()
                .append_transcript(&slug, &transcript)
                .map(|_| "转写已写入笔记".to_string())
        }) {
            Ok(message) => {
                refresh_index();
                tasks::update_progress(&task_id, "done", &message, Some(100), None);
            }
            Err(error) => tasks::update_progress(&task_id, "failed", &error, Some(100), None),
        }
    });

    Ok(task)
}

#[tauri::command]
fn task_status(task_id: String) -> Result<TaskStatus, String> {
    tasks::get(&task_id).ok_or_else(|| format!("task not found: {task_id}"))
}

#[tauri::command]
fn list_tasks() -> Result<Vec<TaskStatus>, String> {
    Ok(tasks::list())
}

#[tauri::command]
fn search_sessions(query: String) -> Result<Vec<SessionDetails>, String> {
    Ok(index::search(&store(), &query)?
        .into_iter()
        .map(|hit| hit.session)
        .collect())
}

#[tauri::command]
fn search_index(query: String) -> Result<Vec<SearchHit>, String> {
    index::search(&store(), &query)
}

#[tauri::command]
fn rebuild_index() -> Result<IndexStats, String> {
    index::rebuild(&store())
}

#[tauri::command]
fn summarize_session(slug: String) -> Result<String, String> {
    let session = store().read_session(&slug)?;
    ai::summarize_notes(&session.notes).or_else(|_| Ok(sessions::offline_summary(&session.notes)))
}

#[tauri::command]
fn export_session_markdown(slug: String) -> Result<String, String> {
    export::export_markdown(&store(), &slug)
}

#[tauri::command]
fn export_session_pdf(slug: String) -> Result<String, String> {
    export::export_pdf(&store(), &slug)
}

#[tauri::command]
fn export_session_xmind(slug: String) -> Result<String, String> {
    export::export_xmind(&store(), &slug)
}

#[tauri::command]
fn load_settings() -> AppSettings {
    settings::load_settings()
}

#[tauri::command]
fn save_settings_cmd(key: String, context_window: u32, token_limit: u32) -> Result<(), String> {
    let s = AppSettings {
        openai_api_key: key,
        context_window,
        token_limit,
    };
    settings::save_settings(&s)
}

#[tauri::command]
fn get_token_report() -> TokenReport {
    tokens::get_report()
}

/// 将图片文件读取为 base64 data URL，用于前端预览
#[tauri::command]
fn load_image_as_data_url(path: String) -> Result<String, String> {
    let data = std::fs::read(&path).map_err(|e| format!("failed to read image: {e}"))?;
    let ext = path.rsplit('.').next().unwrap_or("png").to_lowercase();
    let mime = match ext.as_str() {
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        _ => "image/png",
    };
    Ok(format!("data:{mime};base64,{}", base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &data)))
}

// ========== 认证相关命令 ==========

#[tauri::command]
fn send_email_code(email: String) -> Result<String, String> {
    users::send_email_code(&email)
}

#[tauri::command]
fn send_phone_code(phone: String) -> Result<String, String> {
    users::send_phone_code(&phone)
}

#[tauri::command]
fn verify_code(target: String, code: String) -> Result<(), String> {
    users::verify_code(&target, &code)
}

#[tauri::command]
fn register_email(email: String, password: String, code: String) -> Result<AuthResult, String> {
    match users::register_email(&email, &password, &code) {
        Ok(user) => Ok(AuthResult {
            success: true,
            message: "注册成功".to_string(),
            user: Some(user),
        }),
        Err(e) => Ok(AuthResult {
            success: false,
            message: e,
            user: None,
        }),
    }
}

#[tauri::command]
fn register_phone(phone: String, password: String, code: String) -> Result<AuthResult, String> {
    match users::register_phone(&phone, &password, &code) {
        Ok(user) => Ok(AuthResult {
            success: true,
            message: "注册成功".to_string(),
            user: Some(user),
        }),
        Err(e) => Ok(AuthResult {
            success: false,
            message: e,
            user: None,
        }),
    }
}

#[tauri::command]
fn register_username(username: String, password: String) -> Result<AuthResult, String> {
    match users::register_username(&username, &password) {
        Ok(user) => Ok(AuthResult {
            success: true,
            message: "注册成功".to_string(),
            user: Some(user),
        }),
        Err(e) => Ok(AuthResult {
            success: false,
            message: e,
            user: None,
        }),
    }
}

#[tauri::command]
fn login_email(email: String, password: String) -> AuthResult {
    users::login_email(&email, &password)
}

#[tauri::command]
fn login_phone(phone: String, password: String) -> AuthResult {
    users::login_phone(&phone, &password)
}

#[tauri::command]
fn login_username(identifier: String, password: String) -> AuthResult {
    users::login_username(&identifier, &password)
}

/// 获取第三方 OAuth 授权 URL
#[tauri::command]
fn get_social_auth_url(provider: String) -> Result<String, String> {
    match provider.as_str() {
        "wechat" => {
            let app_id = std::env::var("WECHAT_APP_ID").unwrap_or_default();
            if app_id.is_empty() {
                return Err("微信登录未配置".to_string());
            }
            Ok(format!(
                "https://open.weixin.qq.com/connect/qrconnect?appid={}&redirect_uri={}&response_type=code&scope=snsapi_login",
                app_id,
                std::env::var("WECHAT_REDIRECT_URI").unwrap_or_else(|_| "http://localhost/callback".to_string())
            ))
        }
        "qq" => {
            let app_id = std::env::var("QQ_APP_ID").unwrap_or_default();
            if app_id.is_empty() {
                return Err("QQ 登录未配置".to_string());
            }
            Ok(format!(
                "https://graph.qq.com/oauth2.0/authorize?response_type=code&client_id={}&redirect_uri={}",
                app_id,
                std::env::var("QQ_REDIRECT_URI").unwrap_or_else(|_| "http://localhost/callback".to_string())
            ))
        }
        "alipay" => {
            let app_id = std::env::var("ALIPAY_APP_ID").unwrap_or_default();
            if app_id.is_empty() {
                return Err("支付宝登录未配置".to_string());
            }
            Ok(format!(
                "https://openauth.alipay.com/oauth2/publicAppAuthorize.htm?app_id={}&scope=auth_user&redirect_uri={}",
                app_id,
                std::env::var("ALIPAY_REDIRECT_URI").unwrap_or_else(|_| "http://localhost/callback".to_string())
            ))
        }
        "github" => {
            let client_id = std::env::var("GITHUB_CLIENT_ID").unwrap_or_default();
            if client_id.is_empty() {
                return Err("GitHub 登录未配置".to_string());
            }
            Ok(format!(
                "https://github.com/login/oauth/authorize?client_id={}&scope=user:email",
                client_id
            ))
        }
        _ => Err(format!("不支持的登录方式：{}", provider)),
    }
}

/// 在浏览器中打开 URL
#[tauri::command]
fn open_url(url: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", &url])
            .spawn()
            .map_err(|e| format!("打开浏览器失败：{}", e))?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&url)
            .spawn()
            .map_err(|e| format!("打开浏览器失败：{}", e))?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&url)
            .spawn()
            .map_err(|e| format!("打开浏览器失败：{}", e))?;
    }
    Ok(())
}

/// 第三方社交登录（完整 OAuth 流程）
#[tauri::command]
fn social_login(provider: String) -> Result<OAuthResult, String> {
    oauth::social_login_or_register(&provider)
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            list_sessions,
            create_session,
            read_session,
            save_note,
            capture_session_screenshot,
            analyze_session_screenshot,
            start_session_recording,
            stop_session_recording,
            list_input_devices,
            test_input_device,
            start_session_audio,
            stop_session_audio,
            enqueue_transcribe_latest_audio,
            task_status,
            list_tasks,
            search_sessions,
            search_index,
            rebuild_index,
            summarize_session,
            export_session_markdown,
            export_session_pdf,
            export_session_xmind,
            load_settings,
            save_settings_cmd,
            get_token_report,
            load_image_as_data_url,
            send_email_code,
            send_phone_code,
            verify_code,
            register_email,
            register_phone,
            register_username,
            login_email,
            login_phone,
            login_username,
            get_social_auth_url,
            open_url,
            social_login
        ])
        .run(tauri::generate_context!())
        .expect("failed to run AI Listen RS");
}

pub fn timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    seconds.to_string()
}

fn refresh_index() {
    let _ = index::rebuild(&store());
}

fn transcript_preview(transcript: &str) -> String {
    let mut preview = transcript.trim().chars().take(240).collect::<String>();
    if transcript.trim().chars().count() > 240 {
        preview.push_str("...");
    }
    preview
}

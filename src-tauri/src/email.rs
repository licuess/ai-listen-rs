use lettre::{
    message::header::ContentType,
    transport::smtp::authentication::Credentials,
    Message, SmtpTransport, Transport,
};

/// 通过 SMTP 发送验证码邮件
/// 从设置中读取 SMTP 配置，未配置时返回错误
pub fn send_verification_email(to: &str, code: &str) -> Result<(), String> {
    let _settings = crate::settings::load_settings();

    // 从设置中读取 SMTP 配置（存储在 openai_api_key 字段中作为临时方案）
    // 实际应使用独立的 SMTP 配置字段
    let smtp_host = std::env::var("SMTP_HOST").unwrap_or_default();
    let smtp_port: u16 = std::env::var("SMTP_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(465);
    let smtp_user = std::env::var("SMTP_USER").unwrap_or_default();
    let smtp_pass = std::env::var("SMTP_PASS").unwrap_or_default();

    if smtp_host.is_empty() || smtp_user.is_empty() || smtp_pass.is_empty() {
        return Err("SMTP 未配置，请在 src-tauri/.env 中设置 SMTP_HOST/SMTP_USER/SMTP_PASS".to_string());
    }

    let email = Message::builder()
        .from(format!("AI Listen RS <{}>", smtp_user).parse::<lettre::Address>().map_err(|e| e.to_string())?.into())
        .to(to.parse::<lettre::Address>().map_err(|e| e.to_string())?.into())
        .subject("AI Listen RS - 验证码")
        .header(ContentType::TEXT_PLAIN)
        .body(format!(
            "您好，\n\n您的验证码是：{}\n\n验证码 5 分钟内有效，请勿泄露给他人。\n\nAI Listen RS 团队",
            code
        ))
        .map_err(|e| e.to_string())?;

    let creds = Credentials::new(smtp_user.clone(), smtp_pass);

    let transport = SmtpTransport::builder_dangerous(&smtp_host)
        .port(smtp_port)
        .credentials(creds)
        .build();

    transport.send(&email).map_err(|e| format!("邮件发送失败: {}", e))?;

    Ok(())
}

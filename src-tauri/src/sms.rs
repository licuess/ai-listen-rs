/// 通过短信服务商发送验证码
/// 实际使用时需要接入第三方短信服务（如阿里云、腾讯云短信）
/// 未配置时返回错误，由调用方降级处理
pub fn send_sms(phone: &str, code: &str) -> Result<(), String> {
    let sms_provider = std::env::var("SMS_PROVIDER").unwrap_or_default();
    let sms_access_key = std::env::var("SMS_ACCESS_KEY").unwrap_or_default();
    let sms_secret_key = std::env::var("SMS_SECRET_KEY").unwrap_or_default();

    if sms_provider.is_empty() || sms_access_key.is_empty() || sms_secret_key.is_empty() {
        return Err("短信服务未配置".to_string());
    }

    match sms_provider.as_str() {
        "aliyun" => send_aliyun_sms(phone, code, &sms_access_key, &sms_secret_key),
        "tencent" => send_tencent_sms(phone, code, &sms_access_key, &sms_secret_key),
        _ => Err(format!("不支持的短信服务商: {}", sms_provider)),
    }
}

/// 阿里云短信
fn send_aliyun_sms(
    phone: &str,
    code: &str,
    access_key: &str,
    _secret_key: &str,
) -> Result<(), String> {
    // 阿里云短信 API 调用（需要签名和请求构造）
    // 这里提供框架，实际需要实现完整的签名逻辑
    let sign_name = std::env::var("SMS_SIGN_NAME").unwrap_or_else(|_| "AIListenRS".to_string());
    let template_code =
        std::env::var("SMS_TEMPLATE_CODE").unwrap_or_else(|_| "SMS_000001".to_string());

    // 使用 reqwest 调用阿里云短信 API
    let client = reqwest::blocking::Client::new();
    let _params = serde_json::json!({
        "PhoneNumbers": phone,
        "SignName": sign_name,
        "TemplateCode": template_code,
        "TemplateParam": serde_json::json!({ "code": code }).to_string(),
        "AccessKeyId": access_key,
        "Action": "SendSms",
        "Format": "JSON",
        "Version": "2017-05-25",
        "SignatureMethod": "HMAC-SHA1",
        "Timestamp": chrono_now(),
        "SignatureVersion": "1.0",
    });

    let response = client
        .get("https://dysmsapi.aliyuncs.com/")
        .query(&[
            ("PhoneNumbers", phone),
            ("SignName", &sign_name),
            ("TemplateCode", &template_code),
            ("TemplateParam", &format!("{{\"code\":\"{}\"}}", code)),
            ("AccessKeyId", access_key),
            ("Action", "SendSms"),
            ("Format", "JSON"),
            ("Version", "2017-05-25"),
        ])
        .send()
        .map_err(|e| format!("阿里云短信请求失败: {}", e))?;

    if response.status().is_success() {
        Ok(())
    } else {
        Err(format!("阿里云短信发送失败: {}", response.status()))
    }
}

/// 腾讯云短信
fn send_tencent_sms(
    phone: &str,
    code: &str,
    _access_key: &str,
    _secret_key: &str,
) -> Result<(), String> {
    let sdk_app_id = std::env::var("SMS_SDK_APP_ID").unwrap_or_default();
    let template_id = std::env::var("SMS_TEMPLATE_ID").unwrap_or_default();
    let sign_name = std::env::var("SMS_SIGN_NAME").unwrap_or_else(|_| "AIListenRS".to_string());

    if sdk_app_id.is_empty() || template_id.is_empty() {
        return Err("腾讯云短信配置不完整".to_string());
    }

    // 腾讯云短信 API v3 调用
    let body = serde_json::json!({
        "PhoneNumberSet": [phone],
        "SmsSdkAppId": sdk_app_id,
        "SignName": sign_name,
        "TemplateId": template_id,
        "TemplateParamSet": [code],
    });

    let client = reqwest::blocking::Client::new();
    let response = client
        .post("https://sms.tencentcloudapi.com/")
        .header("Content-Type", "application/json")
        .header("X-TC-Action", "SendSms")
        .header("X-TC-Version", "2021-01-11")
        .header("X-TC-Region", "ap-guangzhou")
        .json(&body)
        .send()
        .map_err(|e| format!("腾讯云短信请求失败: {}", e))?;

    if response.status().is_success() {
        Ok(())
    } else {
        Err(format!("腾讯云短信发送失败: {}", response.status()))
    }
}

fn chrono_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    // UTC 格式: 2024-01-01T00:00:00Z
    // 简化实现
    format!("{}", secs)
}

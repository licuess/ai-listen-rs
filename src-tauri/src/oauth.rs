use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::Mutex;
use std::sync::LazyLock;
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthUser {
    pub provider: String,
    pub provider_user_id: String,
    pub username: String,
    pub email: Option<String>,
    pub avatar: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct OAuthResult {
    pub success: bool,
    pub message: String,
    pub user: Option<crate::users::User>,
}

// 存储待处理的 OAuth 回调
static CALLBACK_LOCK: LazyLock<Mutex<HashMap<String, String>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// 启动本地回调服务器，返回端口号
fn start_callback_server() -> Result<u16, String> {
    let listener = TcpListener::bind("127.0.0.1:0").map_err(|e| e.to_string())?;
    let port = listener.local_addr().map_err(|e| e.to_string())?.port();

    thread::spawn(move || {
        listener.set_nonblocking(false).ok();
        for stream in listener.incoming() {
            if let Ok(mut stream) = stream {
                let mut buffer = [0u8; 4096];
                let n = stream.read(&mut buffer).unwrap_or(0);
                let request = String::from_utf8_lossy(&buffer[..n]);

                // 解析 URL 中的 code 参数
                if let Some(url) = request.lines().next() {
                    let url = url.split_whitespace().nth(1).unwrap_or("");
                    if let Some(query) = url.split('?').nth(1) {
                        let params: HashMap<String, String> = query
                            .split('&')
                            .filter_map(|pair| {
                                let mut parts = pair.splitn(2, '=');
                                Some((parts.next()?.to_string(), parts.next()?.to_string()))
                            })
                            .collect();

                        if let Some(code) = params.get("code") {
                            let mut callbacks = CALLBACK_LOCK.lock().unwrap();
                            callbacks.insert("oauth_code".to_string(), code.clone());
                        }
                    }
                }

                // 返回成功页面
                let response = "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\n\r\n\
                    <html><body style='font-family:sans-serif;text-align:center;padding:50px;'>\
                    <h2>授权成功！</h2><p>请返回 AI Listen RS 应用完成登录。</p>\
                    <script>setTimeout(() => window.close(), 2000);</script>\
                    </body></html>";
                stream.write_all(response.as_bytes()).ok();
                stream.flush().ok();
                break;
            }
        }
    });

    Ok(port)
}

/// 等待 OAuth 回调代码
fn wait_for_code(timeout_secs: u64) -> Result<String, String> {
    let start = std::time::Instant::now();
    loop {
        {
            let callbacks = CALLBACK_LOCK.lock().unwrap();
            if let Some(code) = callbacks.get("oauth_code") {
                let code = code.clone();
                drop(callbacks);
                let mut callbacks = CALLBACK_LOCK.lock().unwrap();
                callbacks.remove("oauth_code");
                return Ok(code);
            }
        }
        if start.elapsed() > Duration::from_secs(timeout_secs) {
            return Err("授权超时，请重试".to_string());
        }
        thread::sleep(Duration::from_millis(500));
    }
}

/// 执行完整的 OAuth 登录流程
pub fn oauth_login(provider: &str) -> Result<OAuthUser, String> {
    let port = start_callback_server()?;
    let redirect_uri = format!("http://127.0.0.1:{}/callback", port);

    let auth_url = match provider {
        "github" => {
            let client_id = std::env::var("GITHUB_CLIENT_ID")
                .map_err(|_| "GitHub 未配置，请在 src-tauri/.env 中设置 GITHUB_CLIENT_ID 和 GITHUB_CLIENT_SECRET（https://github.com/settings/developers）".to_string())?;
            format!(
                "https://github.com/login/oauth/authorize?client_id={}&redirect_uri={}&scope=user:email",
                client_id, redirect_uri
            )
        }
        "qq" => {
            let app_id = std::env::var("QQ_APP_ID")
                .map_err(|_| "QQ 未配置，请在 src-tauri/.env 中设置 QQ_APP_ID 和 QQ_APP_KEY（https://connect.qq.com）".to_string())?;
            format!(
                "https://graph.qq.com/oauth2.0/authorize?response_type=code&client_id={}&redirect_uri={}&scope=get_user_info",
                app_id, redirect_uri
            )
        }
        "wechat" => {
            let app_id = std::env::var("WECHAT_APP_ID")
                .map_err(|_| "微信未配置，请在 src-tauri/.env 中设置 WECHAT_APP_ID 和 WECHAT_APP_SECRET（https://open.weixin.qq.com）".to_string())?;
            format!(
                "https://open.weixin.qq.com/connect/qrconnect?appid={}&redirect_uri={}&response_type=code&scope=snsapi_login",
                app_id, redirect_uri
            )
        }
        "alipay" => {
            let app_id = std::env::var("ALIPAY_APP_ID")
                .map_err(|_| "支付宝未配置，请在 src-tauri/.env 中设置 ALIPAY_APP_ID 和 ALIPAY_PRIVATE_KEY（https://open.alipay.com）".to_string())?;
            format!(
                "https://openauth.alipay.com/oauth2/publicAppAuthorize.htm?app_id={}&scope=auth_user&redirect_uri={}",
                app_id, redirect_uri
            )
        }
        _ => return Err(format!("不支持的登录方式：{}", provider)),
    };

    // 在浏览器中打开授权页面
    open_browser(&auth_url)?;

    // 等待回调
    let code = wait_for_code(120)?;

    // 根据平台交换 token 并获取用户信息
    match provider {
        "github" => github_get_user(&code, &redirect_uri),
        "qq" => qq_get_user(&code, &redirect_uri),
        "wechat" => wechat_get_user(&code, &redirect_uri),
        "alipay" => alipay_get_user(&code, &redirect_uri),
        _ => Err(format!("不支持的登录方式：{}", provider)),
    }
}

/// GitHub: 用 code 换 token，再获取用户信息
fn github_get_user(code: &str, redirect_uri: &str) -> Result<OAuthUser, String> {
    let client_id = std::env::var("GITHUB_CLIENT_ID").unwrap_or_default();
    let client_secret = std::env::var("GITHUB_CLIENT_SECRET").unwrap_or_default();

    if client_id.is_empty() || client_secret.is_empty() {
        return Err("GitHub OAuth 未配置".to_string());
    }

    // 交换 access token
    let client = reqwest::blocking::Client::new();
    let token_resp = client
        .post("https://github.com/login/oauth/access_token")
        .header("Accept", "application/json")
        .json(&serde_json::json!({
            "client_id": client_id,
            "client_secret": client_secret,
            "code": code,
            "redirect_uri": redirect_uri
        }))
        .send()
        .map_err(|e| format!("GitHub token 请求失败: {}", e))?;

    let token_data: serde_json::Value = token_resp
        .json()
        .map_err(|e| format!("GitHub token 解析失败: {}", e))?;

    let access_token = token_data["access_token"]
        .as_str()
        .ok_or("GitHub 未返回 access_token")?;

    // 获取用户信息
    let user_resp = client
        .get("https://api.github.com/user")
        .header("Authorization", format!("token {}", access_token))
        .header("User-Agent", "AI-Listen-RS")
        .send()
        .map_err(|e| format!("GitHub 用户信息请求失败: {}", e))?;

    let user_data: serde_json::Value = user_resp
        .json()
        .map_err(|e| format!("GitHub 用户信息解析失败: {}", e))?;

    Ok(OAuthUser {
        provider: "github".to_string(),
        provider_user_id: user_data["id"].to_string(),
        username: user_data["login"].as_str().unwrap_or("github_user").to_string(),
        email: user_data["email"].as_str().map(|s| s.to_string()),
        avatar: user_data["avatar_url"].as_str().map(|s| s.to_string()),
    })
}

/// QQ: 用 code 换 token，再获取用户信息
fn qq_get_user(code: &str, redirect_uri: &str) -> Result<OAuthUser, String> {
    let app_id = std::env::var("QQ_APP_ID").unwrap_or_default();
    let app_key = std::env::var("QQ_APP_KEY").unwrap_or_default();

    if app_id.is_empty() || app_key.is_empty() {
        return Err("QQ OAuth 未配置".to_string());
    }

    let client = reqwest::blocking::Client::new();

    // 交换 access token
    let token_resp = client
        .get("https://graph.qq.com/oauth2.0/token")
        .query(&[
            ("grant_type", "authorization_code"),
            ("client_id", &app_id),
            ("client_secret", &app_key),
            ("code", code),
            ("redirect_uri", redirect_uri),
            ("fmt", "json"),
        ])
        .send()
        .map_err(|e| format!("QQ token 请求失败: {}", e))?;

    let token_data: serde_json::Value = token_resp
        .json()
        .map_err(|e| format!("QQ token 解析失败: {}", e))?;

    let access_token = token_data["access_token"]
        .as_str()
        .ok_or("QQ 未返回 access_token")?;

    // 获取 openid
    let openid_resp = client
        .get("https://graph.qq.com/oauth2.0/me")
        .query(&[("access_token", access_token), ("fmt", "json")])
        .send()
        .map_err(|e| format!("QQ openid 请求失败: {}", e))?;

    let openid_data: serde_json::Value = openid_resp
        .json()
        .map_err(|e| format!("QQ openid 解析失败: {}", e))?;

    let openid = openid_data["openid"]
        .as_str()
        .ok_or("QQ 未返回 openid")?;

    // 获取用户信息
    let user_resp = client
        .get("https://graph.qq.com/user/get_user_info")
        .query(&[
            ("access_token", access_token),
            ("oauth_consumer_key", &app_id),
            ("openid", openid),
            ("format", "json"),
        ])
        .send()
        .map_err(|e| format!("QQ 用户信息请求失败: {}", e))?;

    let user_data: serde_json::Value = user_resp
        .json()
        .map_err(|e| format!("QQ 用户信息解析失败: {}", e))?;

    Ok(OAuthUser {
        provider: "qq".to_string(),
        provider_user_id: openid.to_string(),
        username: user_data["nickname"].as_str().unwrap_or("qq_user").to_string(),
        email: None,
        avatar: user_data["figureurl_qq_2"].as_str().map(|s| s.to_string()),
    })
}

/// 微信: 用 code 换 token，再获取用户信息
fn wechat_get_user(code: &str, _redirect_uri: &str) -> Result<OAuthUser, String> {
    let app_id = std::env::var("WECHAT_APP_ID").unwrap_or_default();
    let app_secret = std::env::var("WECHAT_APP_SECRET").unwrap_or_default();

    if app_id.is_empty() || app_secret.is_empty() {
        return Err("微信 OAuth 未配置".to_string());
    }

    let client = reqwest::blocking::Client::new();

    // 交换 access token
    let token_resp = client
        .get("https://api.weixin.qq.com/sns/oauth2/access_token")
        .query(&[
            ("appid", &app_id),
            ("secret", &app_secret),
            ("code", &code.to_string()),
            ("grant_type", &"authorization_code".to_string()),
        ])
        .send()
        .map_err(|e| format!("微信 token 请求失败: {}", e))?;

    let token_data: serde_json::Value = token_resp
        .json()
        .map_err(|e| format!("微信 token 解析失败: {}", e))?;

    let access_token = token_data["access_token"]
        .as_str()
        .ok_or("微信未返回 access_token")?;

    let openid = token_data["openid"]
        .as_str()
        .ok_or("微信未返回 openid")?;

    // 获取用户信息
    let user_resp = client
        .get("https://api.weixin.qq.com/sns/userinfo")
        .query(&[
            ("access_token", access_token),
            ("openid", openid),
            ("lang", "zh_CN"),
        ])
        .send()
        .map_err(|e| format!("微信用户信息请求失败: {}", e))?;

    let user_data: serde_json::Value = user_resp
        .json()
        .map_err(|e| format!("微信用户信息解析失败: {}", e))?;

    Ok(OAuthUser {
        provider: "wechat".to_string(),
        provider_user_id: openid.to_string(),
        username: user_data["nickname"].as_str().unwrap_or("wechat_user").to_string(),
        email: None,
        avatar: user_data["headimgurl"].as_str().map(|s| s.to_string()),
    })
}

/// 支付宝: 用 code 换 token，再获取用户信息
fn alipay_get_user(code: &str, _redirect_uri: &str) -> Result<OAuthUser, String> {
    let app_id = std::env::var("ALIPAY_APP_ID").unwrap_or_default();
    let private_key = std::env::var("ALIPAY_PRIVATE_KEY").unwrap_or_default();

    if app_id.is_empty() || private_key.is_empty() {
        return Err("支付宝 OAuth 未配置".to_string());
    }

    // 支付宝 OAuth 流程较复杂，需要签名
    // 这里提供框架，实际需要实现 RSA2 签名
    let client = reqwest::blocking::Client::new();

    let token_resp = client
        .post("https://openapi.alipay.com/gateway.do")
        .form(&[
            ("app_id", app_id.as_str()),
            ("method", "alipay.system.oauth.token"),
            ("format", "JSON"),
            ("charset", "utf-8"),
            ("sign_type", "RSA2"),
            ("code", code),
            ("grant_type", "authorization_code"),
        ])
        .send()
        .map_err(|e| format!("支付宝 token 请求失败: {}", e))?;

    let token_data: serde_json::Value = token_resp
        .json()
        .map_err(|e| format!("支付宝 token 解析失败: {}", e))?;

    let _access_token = token_data["alipay_system_oauth_token_response"]["access_token"]
        .as_str()
        .ok_or("支付宝未返回 access_token")?;

    let user_id = token_data["alipay_system_oauth_token_response"]["user_id"]
        .as_str()
        .ok_or("支付宝未返回 user_id")?;

    Ok(OAuthUser {
        provider: "alipay".to_string(),
        provider_user_id: user_id.to_string(),
        username: format!("alipay_{}", &user_id[user_id.len().saturating_sub(6)..]),
        email: None,
        avatar: None,
    })
}

/// 在浏览器中打开 URL
fn open_browser(url: &str) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", url])
            .spawn()
            .map_err(|e| format!("打开浏览器失败: {}", e))?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(url)
            .spawn()
            .map_err(|e| format!("打开浏览器失败: {}", e))?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(url)
            .spawn()
            .map_err(|e| format!("打开浏览器失败: {}", e))?;
    }
    Ok(())
}

/// 社交登录或注册（如果用户不存在则自动注册）
pub fn social_login_or_register(provider: &str) -> Result<OAuthResult, String> {
    let oauth_user = oauth_login(provider)?;

    // 检查用户是否已存在
    let _lock = crate::users::DB_LOCK_EXTERNAL.lock().unwrap();

    if let Some(user) = crate::users::find_user_by_provider(provider, &oauth_user.provider_user_id) {
        return Ok(OAuthResult {
            success: true,
            message: "登录成功".to_string(),
            user: Some(user),
        });
    }

    // 自动注册新用户
    let new_user = crate::users::User {
        id: crate::users::next_user_id(),
        username: oauth_user.username.clone(),
        email: oauth_user.email.clone(),
        phone: None,
        password_hash: String::new(),
        created_at: crate::timestamp(),
        is_vip: false,
        provider: Some(provider.to_string()),
        provider_user_id: Some(oauth_user.provider_user_id.clone()),
        avatar: oauth_user.avatar.clone(),
    };

    crate::users::insert_user(&new_user)?;

    Ok(OAuthResult {
        success: true,
        message: format!("{} 授权成功，已自动注册", provider),
        user: Some(new_user),
    })
}

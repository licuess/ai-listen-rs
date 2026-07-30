use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::LazyLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub password_hash: String,
    pub created_at: String,
    pub is_vip: bool,
    pub provider: Option<String>,
    pub provider_user_id: Option<String>,
    pub avatar: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationCode {
    pub code: String,
    pub target: String, // email or phone
    pub created_at: u64,
    pub expires_at: u64,
    pub used: bool,
}

#[derive(Debug, Serialize)]
pub struct AuthResult {
    pub success: bool,
    pub message: String,
    pub user: Option<User>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserDatabase {
    pub users: Vec<User>,
    pub next_id: u64,
}

impl Default for UserDatabase {
    fn default() -> Self {
        Self {
            users: Vec::new(),
            next_id: 1,
        }
    }
}

fn db_path() -> PathBuf {
    PathBuf::from("../ai-listen-data/.users.json")
}

pub static DB_LOCK_EXTERNAL: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

pub fn load_db_external() -> UserDatabase {
    load_db()
}

pub fn save_db_external(db: &UserDatabase) -> Result<(), String> {
    save_db(db)
}

static DB_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

// 验证码内存存储（5分钟过期）
static CODES_LOCK: LazyLock<Mutex<HashMap<String, VerificationCode>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

fn load_db() -> UserDatabase {
    let path = db_path();
    if path.exists() {
        fs::read_to_string(&path)
            .ok()
            .and_then(|content| serde_json::from_str(&content).ok())
            .unwrap_or_default()
    } else {
        UserDatabase::default()
    }
}

fn save_db(db: &UserDatabase) -> Result<(), String> {
    let path = db_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let content = serde_json::to_string_pretty(db).map_err(|e| e.to_string())?;
    fs::write(&path, content).map_err(|e| e.to_string())
}

/// 简单哈希（桌面应用本地存储，非安全敏感场景）
fn hash_password(password: &str) -> String {
    let mut hash: u64 = 5381;
    for byte in password.bytes() {
        hash = hash.wrapping_mul(33).wrapping_add(byte as u64);
    }
    // 加盐
    format!("{:x}_ai_listen_salt", hash)
}

fn generate_code() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    format!("{:06}", rng.gen_range(100000..999999))
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// 发送邮箱验证码
pub fn send_email_code(email: &str) -> Result<String, String> {
    // 检查频率限制：同一邮箱 60s 内不能重复发送
    let codes = CODES_LOCK.lock().unwrap();
    if let Some(existing) = codes.get(email) {
        if !existing.used && now_secs() < existing.expires_at {
            let elapsed = now_secs() - existing.created_at;
            if elapsed < 60 {
                return Err(format!("验证码已发送，请 {}s 后重试", 60 - elapsed));
            }
        }
    }
    drop(codes);

    let code = generate_code();
    let now = now_secs();

    // 存储验证码（5分钟有效）
    let mut codes = CODES_LOCK.lock().unwrap();
    codes.insert(
        email.to_string(),
        VerificationCode {
            code: code.clone(),
            target: email.to_string(),
            created_at: now,
            expires_at: now + 300,
            used: false,
        },
    );
    drop(codes);

    // 尝试通过 SMTP 发送
    match crate::email::send_verification_email(email, &code) {
        Ok(_) => Ok("验证码已发送到邮箱".to_string()),
        Err(_) => {
            // SMTP 不可用时，返回验证码供调试（桌面开发模式）
            Ok(format!("验证码：{}（SMTP 未配置，开发模式直接显示）", code))
        }
    }
}

/// 发送手机验证码
pub fn send_phone_code(phone: &str) -> Result<String, String> {
    if phone.len() < 11 {
        return Err("手机号格式不正确".to_string());
    }

    let codes = CODES_LOCK.lock().unwrap();
    if let Some(existing) = codes.get(phone) {
        if !existing.used && now_secs() < existing.expires_at {
            let elapsed = now_secs() - existing.created_at;
            if elapsed < 60 {
                return Err(format!("验证码已发送，请 {}s 后重试", 60 - elapsed));
            }
        }
    }
    drop(codes);

    let code = generate_code();
    let now = now_secs();

    let mut codes = CODES_LOCK.lock().unwrap();
    codes.insert(
        phone.to_string(),
        VerificationCode {
            code: code.clone(),
            target: phone.to_string(),
            created_at: now,
            expires_at: now + 300,
            used: false,
        },
    );
    drop(codes);

    // 尝试通过短信服务商发送
    match crate::sms::send_sms(phone, &code) {
        Ok(_) => Ok("验证码已发送到手机".to_string()),
        Err(_) => {
            // 短信服务不可用时，返回验证码供调试
            Ok(format!("验证码：{}（短信服务未配置，开发模式直接显示）", code))
        }
    }
}

/// 验证验证码
pub fn verify_code(target: &str, code: &str) -> Result<(), String> {
    let mut codes = CODES_LOCK.lock().unwrap();
    let entry = codes
        .get_mut(target)
        .ok_or("未找到验证码，请先获取验证码")?;

    if entry.used {
        return Err("验证码已使用".to_string());
    }
    if now_secs() > entry.expires_at {
        codes.remove(target);
        return Err("验证码已过期".to_string());
    }
    if entry.code != code {
        return Err("验证码错误".to_string());
    }

    entry.used = true;
    Ok(())
}

/// 邮箱注册
pub fn register_email(email: &str, password: &str) -> Result<User, String> {
    if password.len() < 6 || password.len() > 20 {
        return Err("密码长度为6-20位".to_string());
    }

    let _lock = DB_LOCK.lock().unwrap();
    let mut db = load_db();

    // 检查邮箱是否已注册
    if db.users.iter().any(|u| u.email.as_deref() == Some(email)) {
        return Err("该邮箱已注册".to_string());
    }

    let user = User {
        id: format!("user_{}", db.next_id),
        username: email.split('@').next().unwrap_or(email).to_string(),
        email: Some(email.to_string()),
        phone: None,
        password_hash: hash_password(password),
        created_at: crate::timestamp(),
        is_vip: false,
        provider: None,
        provider_user_id: None,
        avatar: None,
    };

    db.next_id += 1;
    db.users.push(user.clone());
    save_db(&db)?;

    Ok(user)
}

/// 手机注册
pub fn register_phone(phone: &str, password: &str) -> Result<User, String> {
    if password.len() < 6 || password.len() > 20 {
        return Err("密码长度为6-20位".to_string());
    }

    let _lock = DB_LOCK.lock().unwrap();
    let mut db = load_db();

    if db.users.iter().any(|u| u.phone.as_deref() == Some(phone)) {
        return Err("该手机号已注册".to_string());
    }

    let user = User {
        id: format!("user_{}", db.next_id),
        username: format!("user_{}", &phone[phone.len().saturating_sub(4)..]),
        email: None,
        phone: Some(phone.to_string()),
        password_hash: hash_password(password),
        created_at: crate::timestamp(),
        is_vip: false,
        provider: None,
        provider_user_id: None,
        avatar: None,
    };

    db.next_id += 1;
    db.users.push(user.clone());
    save_db(&db)?;

    Ok(user)
}

/// 用户名注册
pub fn register_username(username: &str, password: &str) -> Result<User, String> {
    if password.len() < 6 || password.len() > 20 {
        return Err("密码长度为6-20位".to_string());
    }
    if username.len() < 2 || username.len() > 20 {
        return Err("用户名为2-20个字符".to_string());
    }

    let _lock = DB_LOCK.lock().unwrap();
    let mut db = load_db();

    if db.users.iter().any(|u| u.username == username) {
        return Err("该用户名已被占用".to_string());
    }

    let user = User {
        id: format!("user_{}", db.next_id),
        username: username.to_string(),
        email: None,
        phone: None,
        password_hash: hash_password(password),
        created_at: crate::timestamp(),
        is_vip: false,
        provider: None,
        provider_user_id: None,
        avatar: None,
    };

    db.next_id += 1;
    db.users.push(user.clone());
    save_db(&db)?;

    Ok(user)
}

/// 邮箱登录
pub fn login_email(email: &str, password: &str) -> AuthResult {
    let _lock = DB_LOCK.lock().unwrap();
    let db = load_db();

    let user = db
        .users
        .iter()
        .find(|u| u.email.as_deref() == Some(email));

    match user {
        Some(u) if u.password_hash == hash_password(password) => AuthResult {
            success: true,
            message: "登录成功".to_string(),
            user: Some(u.clone()),
        },
        Some(_) => AuthResult {
            success: false,
            message: "密码错误".to_string(),
            user: None,
        },
        None => AuthResult {
            success: false,
            message: "该邮箱未注册".to_string(),
            user: None,
        },
    }
}

/// 手机登录
pub fn login_phone(phone: &str, password: &str) -> AuthResult {
    let _lock = DB_LOCK.lock().unwrap();
    let db = load_db();

    let user = db.users.iter().find(|u| u.phone.as_deref() == Some(phone));

    match user {
        Some(u) if u.password_hash == hash_password(password) => AuthResult {
            success: true,
            message: "登录成功".to_string(),
            user: Some(u.clone()),
        },
        Some(_) => AuthResult {
            success: false,
            message: "密码错误".to_string(),
            user: None,
        },
        None => AuthResult {
            success: false,
            message: "该手机号未注册".to_string(),
            user: None,
        },
    }
}

/// 用户名/手机号登录（统一入口）
pub fn login_username(identifier: &str, password: &str) -> AuthResult {
    let _lock = DB_LOCK.lock().unwrap();
    let db = load_db();

    let user = db.users.iter().find(|u| {
        u.username == identifier || u.phone.as_deref() == Some(identifier)
    });

    match user {
        Some(u) if u.password_hash == hash_password(password) => AuthResult {
            success: true,
            message: "登录成功".to_string(),
            user: Some(u.clone()),
        },
        Some(_) => AuthResult {
            success: false,
            message: "密码错误".to_string(),
            user: None,
        },
        None => AuthResult {
            success: false,
            message: "用户不存在".to_string(),
            user: None,
        },
    }
}

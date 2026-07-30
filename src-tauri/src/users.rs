use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::LazyLock;
use mysql::prelude::*;
use mysql::{Opts, Pool, PooledConn, params};

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
    pub target: String,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAvatar {
    pub id: String,
    pub user_id: String,
    pub file_path: String,
    pub file_name: String,
    pub mime_type: String,
    pub is_system: bool,
    pub created_at: String,
}

// 验证码内存存储（5分钟过期）
static CODES_LOCK: LazyLock<Mutex<HashMap<String, VerificationCode>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

// MySQL 连接池
static DB_POOL: LazyLock<Option<Pool>> = LazyLock::new(|| {
    let url = std::env::var("MYSQL_URL")
        .unwrap_or_else(|_| "mysql://root:liurun1990525@127.0.0.1:3306/ai-listen-rs".to_string());
    match Pool::new(Opts::from_url(&url).ok()?) {
        Ok(pool) => {
            // 初始化表结构
            if let Ok(mut conn) = pool.get_conn() {
                let _ = init_table(&mut conn);
            }
            Some(pool)
        }
        Err(e) => {
            eprintln!("MySQL 连接失败: {}，将使用 JSON 文件降级存储", e);
            None
        }
    }
});

/// 初始化 users 表和 user_avatars 表
fn init_table(conn: &mut PooledConn) -> Result<(), String> {
    conn.query_drop(
        "CREATE TABLE IF NOT EXISTS users (
            id VARCHAR(64) PRIMARY KEY,
            username VARCHAR(64) NOT NULL,
            email VARCHAR(128) DEFAULT NULL,
            phone VARCHAR(20) DEFAULT NULL,
            password_hash VARCHAR(255) NOT NULL DEFAULT '',
            created_at VARCHAR(32) NOT NULL,
            is_vip TINYINT(1) NOT NULL DEFAULT 0,
            provider VARCHAR(32) DEFAULT NULL,
            provider_user_id VARCHAR(128) DEFAULT NULL,
            avatar TEXT DEFAULT NULL,
            UNIQUE KEY uk_username (username),
            UNIQUE KEY uk_email (email),
            UNIQUE KEY uk_phone (phone),
            KEY idx_provider (provider, provider_user_id)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci"
    ).map_err(|e| format!("创建 users 表失败: {}", e))?;

    conn.query_drop(
        "CREATE TABLE IF NOT EXISTS user_avatars (
            id VARCHAR(64) PRIMARY KEY,
            user_id VARCHAR(64) NOT NULL,
            file_path TEXT NOT NULL,
            file_name VARCHAR(128) NOT NULL DEFAULT '',
            mime_type VARCHAR(64) NOT NULL DEFAULT 'image/png',
            is_system TINYINT(1) NOT NULL DEFAULT 0,
            created_at VARCHAR(32) NOT NULL,
            KEY idx_user_id (user_id)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci"
    ).map_err(|e| format!("创建 user_avatars 表失败: {}", e))?;

    // 确保 file_path 字段类型足够大（老表升级，忽略失败）
    let _ = conn.query_drop("ALTER TABLE user_avatars MODIFY COLUMN file_path TEXT NOT NULL");

    // 确保 users 表有 avatar 字段（老表升级，忽略失败）
    let _ = conn.query_drop("ALTER TABLE users ADD COLUMN avatar TEXT DEFAULT NULL");
    // 确保 avatar 字段类型足够大
    let _ = conn.query_drop("ALTER TABLE users MODIFY COLUMN avatar TEXT DEFAULT NULL");

    // ========== 会议相关表 ==========
    init_session_tables(conn)?;

    Ok(())
}

/// 初始化会议相关表（会议/素材/识别/转写/摘要/索引/Token报表）
fn init_session_tables(conn: &mut PooledConn) -> Result<(), String> {
    // 1. 会议主表
    conn.query_drop(
        "CREATE TABLE IF NOT EXISTS sessions (
            id VARCHAR(64) PRIMARY KEY,
            slug VARCHAR(128) NOT NULL,
            title VARCHAR(255) NOT NULL DEFAULT '',
            notes MEDIUMTEXT,
            user_id VARCHAR(64) DEFAULT NULL,
            created_at VARCHAR(32) NOT NULL,
            updated_at VARCHAR(32) DEFAULT NULL,
            UNIQUE KEY uk_slug (slug),
            KEY idx_sessions_user (user_id),
            KEY idx_sessions_created (created_at),
            CONSTRAINT fk_sessions_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE SET NULL
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci"
    ).map_err(|e| format!("创建 sessions 表失败: {}", e))?;

    // 2. 素材表（截图/录屏/音频统一存储）
    conn.query_drop(
        "CREATE TABLE IF NOT EXISTS session_materials (
            id VARCHAR(64) PRIMARY KEY,
            session_id VARCHAR(64) NOT NULL,
            material_type VARCHAR(20) NOT NULL COMMENT '截图:screenshot 录屏:recording 音频:audio',
            file_path TEXT NOT NULL,
            file_name VARCHAR(255) NOT NULL DEFAULT '',
            file_size BIGINT NOT NULL DEFAULT 0,
            duration_secs INT DEFAULT NULL COMMENT '录音/录屏时长(秒)',
            created_at VARCHAR(32) NOT NULL,
            KEY idx_materials_session_type (session_id, material_type),
            KEY idx_materials_created (created_at),
            CONSTRAINT fk_materials_session FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci"
    ).map_err(|e| format!("创建 session_materials 表失败: {}", e))?;

    // 3. 识别结果表（截图OCR/AI识别）
    conn.query_drop(
        "CREATE TABLE IF NOT EXISTS recognitions (
            id VARCHAR(64) PRIMARY KEY,
            session_id VARCHAR(64) NOT NULL,
            material_id VARCHAR(64) DEFAULT NULL COMMENT '关联的截图素材',
            result_text MEDIUMTEXT,
            model VARCHAR(64) NOT NULL DEFAULT '',
            status VARCHAR(20) NOT NULL DEFAULT 'completed' COMMENT 'pending/processing/completed/failed',
            created_at VARCHAR(32) NOT NULL,
            KEY idx_recog_session (session_id),
            KEY idx_recog_material (material_id),
            KEY idx_recog_status (status),
            CONSTRAINT fk_recog_session FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE,
            CONSTRAINT fk_recog_material FOREIGN KEY (material_id) REFERENCES session_materials(id) ON DELETE SET NULL
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci"
    ).map_err(|e| format!("创建 recognitions 表失败: {}", e))?;

    // 4. 转写表（语音转文字）
    conn.query_drop(
        "CREATE TABLE IF NOT EXISTS transcripts (
            id VARCHAR(64) PRIMARY KEY,
            session_id VARCHAR(64) NOT NULL,
            material_id VARCHAR(64) DEFAULT NULL COMMENT '来源音频素材',
            content MEDIUMTEXT,
            language VARCHAR(20) NOT NULL DEFAULT 'zh',
            model VARCHAR(64) NOT NULL DEFAULT '',
            status VARCHAR(20) NOT NULL DEFAULT 'pending' COMMENT 'pending/processing/completed/failed',
            created_at VARCHAR(32) NOT NULL,
            KEY idx_trans_session (session_id),
            KEY idx_trans_material (material_id),
            KEY idx_trans_status (session_id, status),
            CONSTRAINT fk_trans_session FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE,
            CONSTRAINT fk_trans_material FOREIGN KEY (material_id) REFERENCES session_materials(id) ON DELETE SET NULL
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci"
    ).map_err(|e| format!("创建 transcripts 表失败: {}", e))?;

    // 5. 摘要表（AI生成会议摘要）
    conn.query_drop(
        "CREATE TABLE IF NOT EXISTS summaries (
            id VARCHAR(64) PRIMARY KEY,
            session_id VARCHAR(64) NOT NULL,
            content MEDIUMTEXT,
            model VARCHAR(64) NOT NULL DEFAULT '',
            created_at VARCHAR(32) NOT NULL,
            KEY idx_summary_session_created (session_id, created_at),
            CONSTRAINT fk_summary_session FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci"
    ).map_err(|e| format!("创建 summaries 表失败: {}", e))?;

    // 6. 搜索索引表（全文检索）
    conn.query_drop(
        "CREATE TABLE IF NOT EXISTS search_index (
            id BIGINT AUTO_INCREMENT PRIMARY KEY,
            session_id VARCHAR(64) NOT NULL,
            source_type VARCHAR(20) NOT NULL COMMENT 'notes/transcript/recognition/summary',
            content TEXT NOT NULL,
            keywords TEXT,
            updated_at VARCHAR(32) DEFAULT NULL,
            KEY idx_search_session_source (session_id, source_type),
            FULLTEXT KEY ft_search_content (content, keywords) WITH PARSER ngram,
            CONSTRAINT fk_search_session FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci"
    ).map_err(|e| format!("创建 search_index 表失败: {}", e))?;

    // 7. Token用量报表（AI接口调用记录）
    conn.query_drop(
        "CREATE TABLE IF NOT EXISTS token_usage (
            id BIGINT AUTO_INCREMENT PRIMARY KEY,
            user_id VARCHAR(64) DEFAULT NULL,
            session_id VARCHAR(64) DEFAULT NULL,
            operation VARCHAR(32) NOT NULL COMMENT 'recognition/transcribe/summary/chat',
            model VARCHAR(64) NOT NULL DEFAULT '',
            input_tokens INT NOT NULL DEFAULT 0,
            output_tokens INT NOT NULL DEFAULT 0,
            total_tokens INT NOT NULL DEFAULT 0,
            created_at VARCHAR(32) NOT NULL,
            KEY idx_token_user_created (user_id, created_at),
            KEY idx_token_session (session_id),
            KEY idx_token_operation (operation, created_at),
            CONSTRAINT fk_token_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE SET NULL,
            CONSTRAINT fk_token_session FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE SET NULL
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci"
    ).map_err(|e| format!("创建 token_usage 表失败: {}", e))?;

    Ok(())
}

/// 应用启动时初始化数据库连接池（确保表结构被创建）
pub fn init_db_pool() {
    let _ = DB_POOL.as_ref();
}

/// 获取 MySQL 连接
fn get_conn() -> Result<PooledConn, String> {
    DB_POOL.as_ref()
        .ok_or("MySQL 未连接")?
        .get_conn()
        .map_err(|e| format!("获取数据库连接失败: {}", e))
}

/// 从数据库行映射为 User（完全防御性，不会 panic）
fn row_to_user(row: mysql::Row) -> User {
    // 使用 get_opt 安全提取，避免任何 NULL 转换 panic
    let get_str = |name: &str| -> Option<String> {
        row.get_opt(name).and_then(|r| r.ok()).flatten()
    };
    let get_u8 = |name: &str| -> Option<u8> {
        row.get_opt(name).and_then(|r| r.ok()).flatten()
    };

    User {
        id: get_str("id").unwrap_or_default(),
        username: get_str("username").unwrap_or_default(),
        email: get_str("email"),
        phone: get_str("phone"),
        password_hash: get_str("password_hash").unwrap_or_default(),
        created_at: get_str("created_at").unwrap_or_default(),
        is_vip: get_u8("is_vip").map(|v| v != 0).unwrap_or(false),
        provider: get_str("provider"),
        provider_user_id: get_str("provider_user_id"),
        avatar: get_str("avatar"),
    }
}

// ========== JSON 降级存储（MySQL 不可用时） ==========

#[derive(Debug, Serialize, Deserialize)]
struct JsonDatabase {
    users: Vec<User>,
    next_id: u64,
}

impl Default for JsonDatabase {
    fn default() -> Self {
        Self { users: Vec::new(), next_id: 1 }
    }
}

fn json_db_path() -> std::path::PathBuf {
    std::path::PathBuf::from("../ai-listen-data/.users.json")
}

fn load_json_db() -> JsonDatabase {
    let path = json_db_path();
    if path.exists() {
        std::fs::read_to_string(&path)
            .ok()
            .and_then(|c| serde_json::from_str(&c).ok())
            .unwrap_or_default()
    } else {
        JsonDatabase::default()
    }
}

fn save_json_db(db: &JsonDatabase) -> Result<(), String> {
    let path = json_db_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let content = serde_json::to_string_pretty(db).map_err(|e| e.to_string())?;
    std::fs::write(&path, content).map_err(|e| e.to_string())
}

// ========== 公共接口（供 oauth.rs 使用） ==========

pub static DB_LOCK_EXTERNAL: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

/// 通过 provider 查找用户
pub fn find_user_by_provider(provider: &str, provider_user_id: &str) -> Option<User> {
    if let Ok(mut conn) = get_conn() {
        let result: Option<User> = conn
            .exec_first(
                "SELECT * FROM users WHERE provider = :provider AND provider_user_id = :pid",
                params! { "provider" => provider, "pid" => provider_user_id },
            )
            .ok()?
            .map(row_to_user);
        result
    } else {
        let db = load_json_db();
        db.users.into_iter().find(|u| {
            u.provider.as_deref() == Some(provider)
                && u.provider_user_id.as_deref() == Some(provider_user_id)
        })
    }
}

/// 插入新用户（供 oauth 使用）
pub fn insert_user(user: &User) -> Result<(), String> {
    if let Ok(mut conn) = get_conn() {
        conn.exec_drop(
            "INSERT INTO users (id, username, email, phone, password_hash, created_at, is_vip, provider, provider_user_id, avatar)
             VALUES (:id, :username, :email, :phone, :password_hash, :created_at, :is_vip, :provider, :provider_user_id, :avatar)",
            params! {
                "id" => &user.id,
                "username" => &user.username,
                "email" => &user.email,
                "phone" => &user.phone,
                "password_hash" => &user.password_hash,
                "created_at" => &user.created_at,
                "is_vip" => user.is_vip as u8,
                "provider" => &user.provider,
                "provider_user_id" => &user.provider_user_id,
                "avatar" => &user.avatar,
            },
        ).map_err(|e| format!("插入用户失败: {}", e))
    } else {
        let mut db = load_json_db();
        db.users.push(user.clone());
        db.next_id += 1;
        save_json_db(&db)
    }
}

/// 生成下一个用户 ID
pub fn next_user_id() -> String {
    if let Ok(mut conn) = get_conn() {
        let count: Option<i64> = conn
            .query_first("SELECT COUNT(*) as cnt FROM users")
            .ok()
            .flatten();
        format!("user_{}", count.unwrap_or(0) + 1)
    } else {
        let db = load_json_db();
        format!("user_{}", db.next_id)
    }
}

// ========== 工具函数 ==========

fn hash_password(password: &str) -> String {
    let mut hash: u64 = 5381;
    for byte in password.bytes() {
        hash = hash.wrapping_mul(33).wrapping_add(byte as u64);
    }
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

// ========== 验证码 ==========

/// 发送邮箱验证码
pub fn send_email_code(email: &str) -> Result<String, String> {
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

    match crate::email::send_verification_email(email, &code) {
        Ok(_) => Ok("验证码已发送到邮箱".to_string()),
        Err(e) => Ok(format!("验证码：{}（{}，开发模式直接显示）", code, e)),
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

    match crate::sms::send_sms(phone, &code) {
        Ok(_) => Ok("验证码已发送到手机".to_string()),
        Err(_) => Ok(format!("验证码：{}（短信服务未配置，开发模式直接显示）", code)),
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

// ========== 注册 ==========

/// 邮箱注册（需要验证码）
pub fn register_email(email: &str, password: &str, code: &str) -> Result<User, String> {
    if !email.contains('@') || !email.contains('.') {
        return Err("邮箱格式不正确".to_string());
    }
    if password.len() < 6 || password.len() > 20 {
        return Err("密码长度为6-20位".to_string());
    }

    verify_code(email, code)?;

    // 检查邮箱是否已注册
    if let Ok(mut conn) = get_conn() {
        let exists: Option<String> = conn
            .exec_first("SELECT id FROM users WHERE email = :email", params! { "email" => email })
            .map_err(|e| e.to_string())?;
        if exists.is_some() {
            return Err("该邮箱已注册".to_string());
        }
    }

    let user = User {
        id: next_user_id(),
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

    insert_user(&user)?;
    Ok(user)
}

/// 手机注册（需要验证码）
pub fn register_phone(phone: &str, password: &str, code: &str) -> Result<User, String> {
    if phone.len() != 11 || !phone.chars().all(|c| c.is_ascii_digit()) {
        return Err("手机号格式不正确（需11位数字）".to_string());
    }
    if password.len() < 6 || password.len() > 20 {
        return Err("密码长度为6-20位".to_string());
    }

    verify_code(phone, code)?;

    if let Ok(mut conn) = get_conn() {
        let exists: Option<String> = conn
            .exec_first("SELECT id FROM users WHERE phone = :phone", params! { "phone" => phone })
            .map_err(|e| e.to_string())?;
        if exists.is_some() {
            return Err("该手机号已注册".to_string());
        }
    }

    let user = User {
        id: next_user_id(),
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

    insert_user(&user)?;
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
    if !username.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
        return Err("用户名只能包含字母、数字、下划线和连字符".to_string());
    }

    // 检查用户名是否已存在
    if let Ok(mut conn) = get_conn() {
        let exists: Option<String> = conn
            .exec_first("SELECT id FROM users WHERE username = :username", params! { "username" => username })
            .map_err(|e| e.to_string())?;
        if exists.is_some() {
            return Err("该用户名已被占用".to_string());
        }
    } else {
        let db = load_json_db();
        if db.users.iter().any(|u| u.username == username) {
            return Err("该用户名已被占用".to_string());
        }
    }

    let user = User {
        id: next_user_id(),
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

    insert_user(&user)?;
    Ok(user)
}

// ========== 登录 ==========

/// 邮箱登录
pub fn login_email(email: &str, password: &str) -> AuthResult {
    let user = if let Ok(mut conn) = get_conn() {
        let row: Option<mysql::Row> = conn
            .exec_first("SELECT * FROM users WHERE email = :email", params! { "email" => email })
            .ok()
            .flatten();
        row.map(row_to_user)
    } else {
        let db = load_json_db();
        db.users.into_iter().find(|u| u.email.as_deref() == Some(email))
    };

    match user {
        Some(u) if u.password_hash == hash_password(password) => AuthResult {
            success: true,
            message: "登录成功".to_string(),
            user: Some(u),
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
    let user = if let Ok(mut conn) = get_conn() {
        let row: Option<mysql::Row> = conn
            .exec_first("SELECT * FROM users WHERE phone = :phone", params! { "phone" => phone })
            .ok()
            .flatten();
        row.map(row_to_user)
    } else {
        let db = load_json_db();
        db.users.into_iter().find(|u| u.phone.as_deref() == Some(phone))
    };

    match user {
        Some(u) if u.password_hash == hash_password(password) => AuthResult {
            success: true,
            message: "登录成功".to_string(),
            user: Some(u),
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

/// 手机验证码登录（无需密码，未注册则自动注册）
pub fn login_phone_code(phone: &str, code: &str) -> Result<User, String> {
    if phone.len() != 11 || !phone.chars().all(|c| c.is_ascii_digit()) {
        return Err("手机号格式不正确（需11位数字）".to_string());
    }

    verify_code(phone, code)?;

    // 查找已注册用户
    if let Ok(mut conn) = get_conn() {
        let row: Option<mysql::Row> = conn
            .exec_first("SELECT * FROM users WHERE phone = :phone", params! { "phone" => phone })
            .map_err(|e| e.to_string())?;
        if let Some(row) = row {
            return Ok(row_to_user(row));
        }
    } else {
        let db = load_json_db();
        if let Some(user) = db.users.into_iter().find(|u| u.phone.as_deref() == Some(phone)) {
            return Ok(user);
        }
    }

    // 未注册则自动注册
    let user = User {
        id: next_user_id(),
        username: format!("user_{}", &phone[phone.len().saturating_sub(4)..]),
        email: None,
        phone: Some(phone.to_string()),
        password_hash: String::new(),
        created_at: crate::timestamp(),
        is_vip: false,
        provider: None,
        provider_user_id: None,
        avatar: None,
    };

    insert_user(&user)?;
    Ok(user)
}

/// 用户名/手机号登录（统一入口）
pub fn login_username(identifier: &str, password: &str) -> AuthResult {
    let user = if let Ok(mut conn) = get_conn() {
        let row: Option<mysql::Row> = conn
            .exec_first(
                "SELECT * FROM users WHERE username = :id OR phone = :id",
                params! { "id" => identifier },
            )
            .ok()
            .flatten();
        row.map(row_to_user)
    } else {
        let db = load_json_db();
        db.users.into_iter().find(|u| {
            u.username == identifier || u.phone.as_deref() == Some(identifier)
        })
    };

    match user {
        Some(u) if u.password_hash == hash_password(password) => AuthResult {
            success: true,
            message: "登录成功".to_string(),
            user: Some(u),
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

/// 更新用户个人信息
pub fn update_profile(
    user_id: &str,
    username: Option<&str>,
    email: Option<&str>,
    phone: Option<&str>,
    avatar: Option<&str>,
) -> Result<User, String> {
    if let Ok(mut conn) = get_conn() {
        // 先获取当前用户数据
        let row: Option<mysql::Row> = conn
            .exec_first("SELECT * FROM users WHERE id = :id", params! { "id" => user_id })
            .map_err(|e| e.to_string())?;
        let current = row.map(row_to_user).ok_or("用户不存在")?;

        // 合并新值（有则更新，无则保持原值）
        let new_username = match username {
            Some(u) => {
                if u.len() < 2 || u.len() > 20 {
                    return Err("用户名为2-20个字符".to_string());
                }
                u.to_string()
            }
            None => current.username.clone(),
        };
        let new_email = match email {
            Some(e) if !e.is_empty() => Some(e.to_string()),
            Some(_) => None,
            None => current.email.clone(),
        };
        let new_phone = match phone {
            Some(p) if !p.is_empty() => Some(p.to_string()),
            Some(_) => None,
            None => current.phone.clone(),
        };
        let new_avatar = match avatar {
            Some(a) if !a.is_empty() => Some(a.to_string()),
            Some(_) => None,
            None => current.avatar.clone(),
        };

        // 固定 SQL，所有字段都更新，避免动态参数问题
        conn.exec_drop(
            "UPDATE users SET username = :username, email = :email, phone = :phone, avatar = :avatar WHERE id = :id",
            params! {
                "id" => user_id,
                "username" => new_username,
                "email" => new_email,
                "phone" => new_phone,
                "avatar" => new_avatar,
            },
        ).map_err(|e| format!("更新失败: {}", e))?;

        // 返回更新后的用户
        let row: Option<mysql::Row> = conn
            .exec_first("SELECT * FROM users WHERE id = :id", params! { "id" => user_id })
            .map_err(|e| e.to_string())?;
        row.map(row_to_user).ok_or("用户不存在".to_string())
    } else {
        // JSON 降级存储
        let mut db = load_json_db();
        let user = db.users.iter_mut().find(|u| u.id == user_id)
            .ok_or("用户不存在")?;
        if let Some(u) = username { user.username = u.to_string(); }
        if let Some(e) = email { user.email = if e.is_empty() { None } else { Some(e.to_string()) }; }
        if let Some(p) = phone { user.phone = if p.is_empty() { None } else { Some(p.to_string()) }; }
        if let Some(a) = avatar { user.avatar = if a.is_empty() { None } else { Some(a.to_string()) }; }
        let updated = user.clone();
        save_json_db(&db)?;
        Ok(updated)
    }
}

// ========== 头像管理 ==========

/// 头像存储目录
fn avatars_dir() -> std::path::PathBuf {
    std::path::PathBuf::from("../ai-listen-data/avatars")
}

/// 上传头像（保存文件 + 写入 user_avatars 表 + 更新 users.avatar）
pub fn upload_avatar(user_id: &str, file_data: &[u8], file_name: &str) -> Result<UserAvatar, String> {
    // 确保目录存在
    let dir = avatars_dir().join(user_id);
    std::fs::create_dir_all(&dir).map_err(|e| format!("创建头像目录失败: {}", e))?;

    // 生成文件名
    let ext = file_name.rsplit('.').next().unwrap_or("png");
    let stored_name = format!("{}.{}", crate::timestamp(), ext);
    let file_path = dir.join(&stored_name);

    // 写入文件
    std::fs::write(&file_path, file_data).map_err(|e| format!("保存头像文件失败: {}", e))?;

    let path_str = file_path.to_string_lossy().to_string();
    let mime = match ext.to_lowercase().as_str() {
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        _ => "image/png",
    };

    let avatar = UserAvatar {
        id: format!("avatar_{}", crate::timestamp()),
        user_id: user_id.to_string(),
        file_path: path_str.clone(),
        file_name: file_name.to_string(),
        mime_type: mime.to_string(),
        is_system: false,
        created_at: crate::timestamp(),
    };

    // 写入数据库
    if let Ok(mut conn) = get_conn() {
        conn.exec_drop(
            "INSERT INTO user_avatars (id, user_id, file_path, file_name, mime_type, is_system, created_at) VALUES (:id, :user_id, :file_path, :file_name, :mime, :sys, :created)",
            params! {
                "id" => avatar.id.clone(),
                "user_id" => avatar.user_id.clone(),
                "file_path" => avatar.file_path.clone(),
                "file_name" => avatar.file_name.clone(),
                "mime" => avatar.mime_type.clone(),
                "sys" => 0u8,
                "created" => avatar.created_at.clone(),
            },
        ).map_err(|e| format!("写入 user_avatars 表失败: {}", e))?;
        // 更新 users.avatar 为当前图片地址
        let _ = conn.exec_drop(
            "UPDATE users SET avatar = :avatar WHERE id = :id",
            params! { "id" => user_id, "avatar" => path_str },
        );
    }

    Ok(avatar)
}

/// 设置用户头像（选择已有头像或系统头像）
pub fn set_user_avatar(user_id: &str, avatar_path: &str) -> Result<User, String> {
    if let Ok(mut conn) = get_conn() {
        conn.exec_drop(
            "UPDATE users SET avatar = :avatar WHERE id = :id",
            params! { "id" => user_id, "avatar" => avatar_path },
        ).map_err(|e| format!("设置头像失败: {}", e))?;

        let row: Option<mysql::Row> = conn
            .exec_first("SELECT * FROM users WHERE id = :id", params! { "id" => user_id })
            .map_err(|e| e.to_string())?;
        row.map(row_to_user).ok_or("用户不存在".to_string())
    } else {
        let mut db = load_json_db();
        let user = db.users.iter_mut().find(|u| u.id == user_id)
            .ok_or("用户不存在")?;
        user.avatar = Some(avatar_path.to_string());
        let updated = user.clone();
        save_json_db(&db)?;
        Ok(updated)
    }
}

/// 记录系统表情头像（写入 user_avatars 表 is_system=1 + 更新 users.avatar）
pub fn record_system_avatar(user_id: &str, emoji_json: &str) -> Result<UserAvatar, String> {
    let avatar = UserAvatar {
        id: format!("avatar_{}", crate::timestamp()),
        user_id: user_id.to_string(),
        file_path: emoji_json.to_string(),
        file_name: "system_emoji".to_string(),
        mime_type: "system/emoji".to_string(),
        is_system: true,
        created_at: crate::timestamp(),
    };

    if let Ok(mut conn) = get_conn() {
        conn.exec_drop(
            "INSERT INTO user_avatars (id, user_id, file_path, file_name, mime_type, is_system, created_at) VALUES (:id, :user_id, :file_path, :file_name, :mime, :sys, :created)",
            params! {
                "id" => avatar.id.clone(),
                "user_id" => avatar.user_id.clone(),
                "file_path" => avatar.file_path.clone(),
                "file_name" => avatar.file_name.clone(),
                "mime" => avatar.mime_type.clone(),
                "sys" => 1u8,
                "created" => avatar.created_at.clone(),
            },
        ).map_err(|e| format!("写入 user_avatars 表失败: {}", e))?;
        // 更新 users.avatar 为表情 JSON
        let _ = conn.exec_drop(
            "UPDATE users SET avatar = :avatar WHERE id = :id",
            params! { "id" => user_id, "avatar" => emoji_json },
        );
    }

    Ok(avatar)
}

/// 记录自定义上传头像（与系统头像相同的 JSON 格式存储，写入 user_avatars 表 is_system=0 + 更新 users.avatar）
pub fn record_custom_avatar(user_id: &str, avatar_json: &str) -> Result<UserAvatar, String> {
    let avatar = UserAvatar {
        id: format!("avatar_{}", crate::timestamp()),
        user_id: user_id.to_string(),
        file_path: avatar_json.to_string(),
        file_name: "custom_upload".to_string(),
        mime_type: "image/custom".to_string(),
        is_system: false,
        created_at: crate::timestamp(),
    };

    if let Ok(mut conn) = get_conn() {
        conn.exec_drop(
            "INSERT INTO user_avatars (id, user_id, file_path, file_name, mime_type, is_system, created_at) VALUES (:id, :user_id, :file_path, :file_name, :mime, :sys, :created)",
            params! {
                "id" => avatar.id.clone(),
                "user_id" => avatar.user_id.clone(),
                "file_path" => avatar.file_path.clone(),
                "file_name" => avatar.file_name.clone(),
                "mime" => avatar.mime_type.clone(),
                "sys" => 0u8,
                "created" => avatar.created_at.clone(),
            },
        ).map_err(|e| format!("写入 user_avatars 表失败: {}", e))?;
        // 更新 users.avatar 为自定义头像 JSON（与系统头像格式一致）
        let _ = conn.exec_drop(
            "UPDATE users SET avatar = :avatar WHERE id = :id",
            params! { "id" => user_id, "avatar" => avatar_json },
        );
    }

    Ok(avatar)
}

/// 获取用户的所有头像列表
pub fn list_user_avatars(user_id: &str) -> Result<Vec<UserAvatar>, String> {
    if let Ok(mut conn) = get_conn() {
        let rows: Vec<mysql::Row> = conn
            .exec(
                "SELECT * FROM user_avatars WHERE user_id = :uid ORDER BY created_at DESC",
                params! { "uid" => user_id },
            )
            .map_err(|e| e.to_string())?;
        Ok(rows.into_iter().map(|row| {
            let get_str = |name: &str| -> Option<String> {
                row.get_opt(name).and_then(|r| r.ok()).flatten()
            };
            let get_u8 = |name: &str| -> Option<u8> {
                row.get_opt(name).and_then(|r| r.ok()).flatten()
            };
            UserAvatar {
                id: get_str("id").unwrap_or_default(),
                user_id: get_str("user_id").unwrap_or_default(),
                file_path: get_str("file_path").unwrap_or_default(),
                file_name: get_str("file_name").unwrap_or_default(),
                mime_type: get_str("mime_type").unwrap_or_else(|| "image/png".to_string()),
                is_system: get_u8("is_system").map(|v| v != 0).unwrap_or(false),
                created_at: get_str("created_at").unwrap_or_default(),
            }
        }).collect())
    } else {
        Ok(Vec::new())
    }
}

/// 删除指定的头像记录（批量）
pub fn delete_avatar_records(user_id: &str, avatar_ids: &[String]) -> Result<u64, String> {
    if avatar_ids.is_empty() {
        return Ok(0);
    }
    if let Ok(mut conn) = get_conn() {
        let mut deleted = 0u64;
        for id in avatar_ids {
            conn.exec_drop(
                "DELETE FROM user_avatars WHERE id = :id AND user_id = :uid",
                params! { "id" => id.as_str(), "uid" => user_id },
            ).map_err(|e| format!("删除头像记录失败: {}", e))?;
            deleted += 1;
        }
        Ok(deleted)
    } else {
        Err("数据库连接失败".to_string())
    }
}

/// 设置当前使用的头像（更新 users.avatar）
pub fn apply_avatar_record(user_id: &str, avatar_json: &str) -> Result<User, String> {
    set_user_avatar(user_id, avatar_json)
}

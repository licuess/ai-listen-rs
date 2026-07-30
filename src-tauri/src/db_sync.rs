//! 数据库同步模块：将本地文件操作同步到 MySQL 数据库
//! 所有函数为 fire-and-forget 模式，DB 不可用时不影响业务操作

use mysql::params;

/// 获取数据库连接（复用 users 模块的连接池）
fn get_conn() -> Result<mysql::PooledConn, String> {
    crate::users::get_db_conn()
}

/// 根据 slug 获取数据库中的 session id（不存在则自动创建）
fn ensure_session_id(conn: &mut mysql::PooledConn, slug: &str, title: &str) -> Option<String> {
    let row: Option<(String,)> = conn
        .exec_first("SELECT id FROM sessions WHERE slug = :slug", params! { "slug" => slug })
        .ok()?;
    if let Some((id,)) = row {
        return Some(id);
    }
    // 不存在则创建（兼容旧的文件会话）
    let id = format!("session_{}", crate::timestamp());
    conn.exec_drop(
        "INSERT IGNORE INTO sessions (id, slug, title, notes, user_id, created_at, updated_at) VALUES (:id, :slug, :title, '', NULL, :created, :updated)",
        params! {
            "id" => id.clone(),
            "slug" => slug,
            "title" => title,
            "created" => crate::timestamp(),
            "updated" => crate::timestamp(),
        },
    ).ok()?;
    Some(id)
}

/// 同步：会议创建
pub fn on_session_created(slug: &str, title: &str, notes: &str) {
    let Ok(mut conn) = get_conn() else { return };
    let id = format!("session_{}", crate::timestamp());
    let _ = conn.exec_drop(
        "INSERT INTO sessions (id, slug, title, notes, user_id, created_at, updated_at) VALUES (:id, :slug, :title, :notes, NULL, :created, :updated) ON DUPLICATE KEY UPDATE title = :title2, notes = :notes2, updated_at = :updated2",
        params! {
            "id" => id,
            "slug" => slug,
            "title" => title,
            "notes" => notes,
            "created" => crate::timestamp(),
            "updated" => crate::timestamp(),
            "title2" => title,
            "notes2" => notes,
            "updated2" => crate::timestamp(),
        },
    );
}

/// 同步：会议删除
pub fn on_session_deleted(slug: &str) {
    let Ok(mut conn) = get_conn() else { return };
    let _ = conn.exec_drop("DELETE FROM sessions WHERE slug = :slug", params! { "slug" => slug });
}

/// 同步：会议笔记更新
pub fn on_notes_updated(slug: &str, title: &str, notes: &str) {
    let Ok(mut conn) = get_conn() else { return };
    if ensure_session_id(&mut conn, slug, title).is_none() { return; }
    let _ = conn.exec_drop(
        "UPDATE sessions SET notes = :notes, title = :title, updated_at = :updated WHERE slug = :slug",
        params! {
            "slug" => slug,
            "title" => title,
            "notes" => notes,
            "updated" => crate::timestamp(),
        },
    );
}

/// 同步：新增素材（截图/录屏/音频）
pub fn on_material_added(slug: &str, title: &str, material_type: &str, file_path: &str, file_name: &str) {
    let Ok(mut conn) = get_conn() else { return };
    let Some(session_id) = ensure_session_id(&mut conn, slug, title) else { return };
    let file_size = std::fs::metadata(file_path).map(|m| m.len() as i64).unwrap_or(0);
    let _ = conn.exec_drop(
        "INSERT INTO session_materials (id, session_id, material_type, file_path, file_name, file_size, duration_secs, created_at) VALUES (:id, :sid, :mtype, :path, :name, :size, NULL, :created)",
        params! {
            "id" => format!("mat_{}", crate::timestamp()),
            "sid" => session_id,
            "mtype" => material_type,
            "path" => file_path,
            "name" => file_name,
            "size" => file_size,
            "created" => crate::timestamp(),
        },
    );
}

/// 同步：识别结果
pub fn on_recognition(slug: &str, title: &str, material_path: Option<&str>, result_text: &str, model: &str) {
    let Ok(mut conn) = get_conn() else { return };
    let Some(session_id) = ensure_session_id(&mut conn, slug, title) else { return };
    // 查找关联的素材 id
    let material_id: Option<String> = material_path.and_then(|path| {
        conn.exec_first(
            "SELECT id FROM session_materials WHERE session_id = :sid AND file_path = :path ORDER BY created_at DESC LIMIT 1",
            params! { "sid" => session_id.clone(), "path" => path },
        ).ok().flatten().map(|(id,): (String,)| id)
    });
    let _ = conn.exec_drop(
        "INSERT INTO recognitions (id, session_id, material_id, result_text, model, status, created_at) VALUES (:id, :sid, :mid, :text, :model, 'completed', :created)",
        params! {
            "id" => format!("recog_{}", crate::timestamp()),
            "sid" => session_id,
            "mid" => material_id,
            "text" => result_text,
            "model" => model,
            "created" => crate::timestamp(),
        },
    );
}

/// 同步：转写结果
pub fn on_transcript(slug: &str, title: &str, material_path: Option<&str>, content: &str, model: &str) {
    let Ok(mut conn) = get_conn() else { return };
    let Some(session_id) = ensure_session_id(&mut conn, slug, title) else { return };
    let material_id: Option<String> = material_path.and_then(|path| {
        conn.exec_first(
            "SELECT id FROM session_materials WHERE session_id = :sid AND file_path = :path ORDER BY created_at DESC LIMIT 1",
            params! { "sid" => session_id.clone(), "path" => path },
        ).ok().flatten().map(|(id,): (String,)| id)
    });
    let _ = conn.exec_drop(
        "INSERT INTO transcripts (id, session_id, material_id, content, language, model, status, created_at) VALUES (:id, :sid, :mid, :content, 'zh', :model, 'completed', :created)",
        params! {
            "id" => format!("trans_{}", crate::timestamp()),
            "sid" => session_id,
            "mid" => material_id,
            "content" => content,
            "model" => model,
            "created" => crate::timestamp(),
        },
    );
}

/// 同步：摘要
pub fn on_summary(slug: &str, title: &str, content: &str, model: &str) {
    let Ok(mut conn) = get_conn() else { return };
    let Some(session_id) = ensure_session_id(&mut conn, slug, title) else { return };
    let _ = conn.exec_drop(
        "INSERT INTO summaries (id, session_id, content, model, created_at) VALUES (:id, :sid, :content, :model, :created)",
        params! {
            "id" => format!("sum_{}", crate::timestamp()),
            "sid" => session_id,
            "content" => content,
            "model" => model,
            "created" => crate::timestamp(),
        },
    );
}

/// 同步：重建搜索索引（全量同步到 search_index 表）
pub fn on_index_rebuilt(sessions: &[(String, String, String)]) {
    // sessions: (slug, title, notes)
    let Ok(mut conn) = get_conn() else { return };
    let _ = conn.query_drop("DELETE FROM search_index");
    for (slug, title, notes) in sessions {
        let Some(session_id) = ensure_session_id(&mut conn, slug, title) else { continue };
        if notes.trim().is_empty() { continue; }
        let _ = conn.exec_drop(
            "INSERT INTO search_index (session_id, source_type, content, keywords, updated_at) VALUES (:sid, 'notes', :content, :keywords, :updated)",
            params! {
                "sid" => session_id,
                "content" => notes.as_str(),
                "keywords" => title.as_str(),
                "updated" => crate::timestamp(),
            },
        );
    }
}

/// 同步：Token 用量
pub fn on_token_usage(operation: &str, slug: Option<&str>, model: &str, input_tokens: u32, output_tokens: u32) {
    let Ok(mut conn) = get_conn() else { return };
    // 查找 session id（如果有 slug）
    let session_id: Option<String> = slug.and_then(|s| {
        conn.exec_first("SELECT id FROM sessions WHERE slug = :slug", params! { "slug" => s })
            .ok().flatten().map(|(id,): (String,)| id)
    });
    let _ = conn.exec_drop(
        "INSERT INTO token_usage (user_id, session_id, operation, model, input_tokens, output_tokens, total_tokens, created_at) VALUES (NULL, :sid, :op, :model, :input, :output, :total, :created)",
        params! {
            "sid" => session_id,
            "op" => operation,
            "model" => model,
            "input" => input_tokens,
            "output" => output_tokens,
            "total" => input_tokens + output_tokens,
            "created" => crate::timestamp(),
        },
    );
}

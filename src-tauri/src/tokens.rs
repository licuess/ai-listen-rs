use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::LazyLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenRecord {
    pub timestamp: String,
    pub operation: String,
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Serialize)]
pub struct TokenReport {
    pub total_prompt_tokens: u32,
    pub total_completion_tokens: u32,
    pub total_tokens: u32,
    pub by_operation: Vec<OperationUsage>,
    pub recent: Vec<TokenRecord>,
}

#[derive(Debug, Serialize)]
pub struct OperationUsage {
    pub operation: String,
    pub count: u32,
    pub total_tokens: u32,
}

fn tokens_path() -> PathBuf {
    PathBuf::from("../ai-listen-data/.tokens.json")
}

static TOKENS_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

pub fn record_usage(operation: &str, prompt_tokens: u32, completion_tokens: u32) {
    let record = TokenRecord {
        timestamp: crate::timestamp(),
        operation: operation.to_string(),
        prompt_tokens,
        completion_tokens,
        total_tokens: prompt_tokens + completion_tokens,
    };

    let _lock = TOKENS_LOCK.lock().unwrap();
    let path = tokens_path();

    let mut records: Vec<TokenRecord> = if path.exists() {
        fs::read_to_string(&path)
            .ok()
            .and_then(|content| serde_json::from_str(&content).ok())
            .unwrap_or_default()
    } else {
        Vec::new()
    };

    records.push(record);

    // 只保留最近 500 条
    if records.len() > 500 {
        records = records.split_off(records.len() - 500);
    }

    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(content) = serde_json::to_string_pretty(&records) {
        let _ = fs::write(&path, content);
    }
}

pub fn get_report() -> TokenReport {
    let _lock = TOKENS_LOCK.lock().unwrap();
    let path = tokens_path();

    let records: Vec<TokenRecord> = if path.exists() {
        fs::read_to_string(&path)
            .ok()
            .and_then(|content| serde_json::from_str(&content).ok())
            .unwrap_or_default()
    } else {
        Vec::new()
    };

    let total_prompt_tokens: u32 = records.iter().map(|r| r.prompt_tokens).sum();
    let total_completion_tokens: u32 = records.iter().map(|r| r.completion_tokens).sum();
    let total_tokens: u32 = records.iter().map(|r| r.total_tokens).sum();

    // 按操作类型聚合
    let mut op_map: std::collections::HashMap<String, (u32, u32)> = std::collections::HashMap::new();
    for r in &records {
        let entry = op_map.entry(r.operation.clone()).or_insert((0, 0));
        entry.0 += 1;
        entry.1 += r.total_tokens;
    }
    let mut by_operation: Vec<OperationUsage> = op_map
        .into_iter()
        .map(|(operation, (count, total_tokens))| OperationUsage {
            operation,
            count,
            total_tokens,
        })
        .collect();
    by_operation.sort_by(|a, b| b.total_tokens.cmp(&a.total_tokens));

    // 最近 20 条
    let recent: Vec<TokenRecord> = records.iter().rev().take(20).cloned().collect();

    TokenReport {
        total_prompt_tokens,
        total_completion_tokens,
        total_tokens,
        by_operation,
        recent,
    }
}

/// 从 API 响应中提取 token 用量
pub fn extract_usage_from_response(value: &serde_json::Value) -> Option<(u32, u32)> {
    if let Some(usage) = value.get("usage") {
        let prompt = usage.get("prompt_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
        let completion = usage.get("completion_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
        if prompt > 0 || completion > 0 {
            return Some((prompt, completion));
        }
    }
    // 也检查 input_tokens / output_tokens 格式
    if let Some(usage) = value.get("usage") {
        let prompt = usage.get("input_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
        let completion = usage.get("output_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
        if prompt > 0 || completion > 0 {
            return Some((prompt, completion));
        }
    }
    None
}

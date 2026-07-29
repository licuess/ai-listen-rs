use serde_json::json;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

use crate::sessions::SessionStore;

/// 导出 Markdown 笔记
pub fn export_markdown(store: &SessionStore, slug: &str) -> Result<String, String> {
    let session = store.read_session(slug)?;
    let notes = session.notes;
    if notes.trim().is_empty() {
        return Err("笔记为空，无法导出".to_string());
    }

    let output = export_path(slug, "md");
    fs::write(&output, &notes).map_err(|error| format!("failed to write markdown: {error}"))?;
    Ok(output.to_string_lossy().to_string())
}

/// 导出 PDF（通过 WebView 打印方案，这里先生成 HTML）
pub fn export_pdf(store: &SessionStore, slug: &str) -> Result<String, String> {
    let session = store.read_session(slug)?;
    let notes = session.notes;
    if notes.trim().is_empty() {
        return Err("笔记为空，无法导出".to_string());
    }

    let html = markdown_to_html(&notes);
    let output = export_path(slug, "pdf");
    fs::write(&output, html.as_bytes()).map_err(|error| format!("failed to write pdf: {error}"))?;
    Ok(output.to_string_lossy().to_string())
}

/// 导出 XMind 思维导图
pub fn export_xmind(store: &SessionStore, slug: &str) -> Result<String, String> {
    let session = store.read_session(slug)?;
    let notes = session.notes;
    if notes.trim().is_empty() {
        return Err("笔记为空，无法导出".to_string());
    }

    let mindmap = parse_markdown_to_mindmap(&notes);
    let content_json = serde_json::to_string(&mindmap)
        .map_err(|error| format!("failed to serialize xmind content: {error}"))?;

    let output = export_path(slug, "xmind");
    let file = File::create(&output).map_err(|error| format!("failed to create xmind file: {error}"))?;
    let mut zip = ZipWriter::new(file);

    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    zip.start_file("content.json", options)
        .map_err(|error| format!("failed to add content.json to zip: {error}"))?;
    zip.write_all(content_json.as_bytes())
        .map_err(|error| format!("failed to write content.json: {error}"))?;

    // 添加 metadata.json
    let metadata = json!({
        "creator": { "name": "AI Listen RS", "version": "0.1.0" }
    });
    let metadata_json = serde_json::to_string(&metadata).unwrap_or_default();

    zip.start_file("metadata.json", options)
        .map_err(|error| format!("failed to add metadata.json to zip: {error}"))?;
    zip.write_all(metadata_json.as_bytes())
        .map_err(|error| format!("failed to write metadata.json: {error}"))?;

    zip.finish().map_err(|error| format!("failed to finalize xmind zip: {error}"))?;

    Ok(output.to_string_lossy().to_string())
}

fn export_path(slug: &str, ext: &str) -> PathBuf {
    let store = crate::store();
    let session_dir = store.session_dir_by_slug(slug).unwrap();
    session_dir.join(format!("export-{}.{}", crate::timestamp(), ext))
}

/// 简单 Markdown 转 HTML
fn markdown_to_html(markdown: &str) -> String {
    let mut html = String::from(
        r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
<meta charset="UTF-8">
<title>会议笔记</title>
<style>
body { font-family: "Microsoft YaHei", "PingFang SC", sans-serif; max-width: 800px; margin: 40px auto; padding: 0 20px; line-height: 1.8; color: #333; }
h1 { border-bottom: 2px solid #236d5c; padding-bottom: 10px; color: #236d5c; }
h2 { color: #2d7a68; margin-top: 30px; }
h3 { color: #3d8a78; }
ul { padding-left: 24px; }
li { margin: 6px 0; }
p { margin: 12px 0; }
code { background: #f5f5f5; padding: 2px 6px; border-radius: 4px; font-size: 0.9em; }
</style>
</head>
<body>
"#,
    );

    for line in markdown.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("# ") {
            html.push_str(&format!("<h1>{}</h1>\n", &trimmed[2..]));
        } else if trimmed.starts_with("## ") {
            html.push_str(&format!("<h2>{}</h2>\n", &trimmed[3..]));
        } else if trimmed.starts_with("### ") {
            html.push_str(&format!("<h3>{}</h3>\n", &trimmed[4..]));
        } else if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
            html.push_str(&format!("<li>{}</li>\n", &trimmed[2..]));
        } else if trimmed.is_empty() {
            html.push_str("<br>\n");
        } else {
            html.push_str(&format!("<p>{}</p>\n", trimmed));
        }
    }

    html.push_str("</body></html>");
    html
}

/// 解析 Markdown 为 XMind 思维导图结构
fn parse_markdown_to_mindmap(markdown: &str) -> serde_json::Value {
    let mut root_title = "会议主题".to_string();
    let mut current_h2: Option<serde_json::Value> = None;
    let mut current_h3: Option<serde_json::Value> = None;
    let mut h2_children: Vec<serde_json::Value> = Vec::new();
    let mut h3_children: Vec<serde_json::Value> = Vec::new();

    for line in markdown.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("# ") {
            root_title = trimmed[2..].to_string();
        } else if trimmed.starts_with("## ") {
            // 保存上一个 h2 节点
            if let Some(mut h2_node) = current_h2.take() {
                if !h3_children.is_empty() {
                    if let Some(mut h3_node) = current_h3.take() {
                        h3_node["children"] = json!({ "attached": h3_children.clone() });
                        if let Some(arr) = h2_node["children"]["attached"].as_array_mut() {
                            arr.push(h3_node);
                        }
                    }
                }
                h2_children.push(h2_node);
            }
            h3_children.clear();
            current_h3 = None;

            current_h2 = Some(json!({
                "id": format!("h2-{}", h2_children.len()),
                "title": &trimmed[3..],
                "children": { "attached": [] }
            }));
        } else if trimmed.starts_with("### ") {
            // 保存上一个 h3 节点
            if let Some(mut h3_node) = current_h3.take() {
                if !h3_children.is_empty() {
                    h3_node["children"] = json!({ "attached": h3_children.clone() });
                    h3_children.clear();
                }
                if let Some(h2) = &mut current_h2 {
                    if let Some(arr) = h2["children"]["attached"].as_array_mut() {
                        arr.push(h3_node);
                    }
                }
            }

            current_h3 = Some(json!({
                "id": format!("h3-{}-{}", h2_children.len(), h3_children.len()),
                "title": &trimmed[4..],
                "children": { "attached": [] }
            }));
        } else if (trimmed.starts_with("- ") || trimmed.starts_with("* ")) && current_h3.is_some() {
            h3_children.push(json!({
                "id": format!("li-{}-{}-{}", h2_children.len(), current_h3.as_ref().map(|_| 0).unwrap_or(0), h3_children.len()),
                "title": &trimmed[2..]
            }));
        }
    }

    // 处理最后一个 h3
    if let Some(mut h3_node) = current_h3.take() {
        if !h3_children.is_empty() {
            h3_node["children"] = json!({ "attached": h3_children });
        }
        if let Some(h2) = &mut current_h2 {
            if let Some(arr) = h2["children"]["attached"].as_array_mut() {
                arr.push(h3_node);
            }
        }
    }

    // 处理最后一个 h2
    if let Some(h2_node) = current_h2.take() {
        h2_children.push(h2_node);
    }

    json!([{
        "id": "sheet-1",
        "class": "sheet",
        "title": "会议纪要",
        "rootTopic": {
            "id": "root",
            "title": root_title,
            "children": { "attached": h2_children }
        }
    }])
}

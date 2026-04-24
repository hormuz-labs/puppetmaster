use reqwest::Client;
use serde_json::{json, Value};
use crate::markdown::markdown_to_telegram_html_chunks;

pub fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

pub fn render_html_chunks(thinking: &str, answer: &str, thinking_header: &str) -> Vec<String> {
    let mut chunks = Vec::new();
    
    let thinking = thinking.trim();
    if !thinking.is_empty() {
        let mut current = String::new();
        for line in thinking.split_inclusive('\n') {
            if current.len() + line.len() > 3800 {
                chunks.push(format!("<blockquote><b>{}</b>\n{}</blockquote>", thinking_header, escape_html(&current)));
                current.clear();
            }
            current.push_str(line);
        }
        if !current.is_empty() {
            chunks.push(format!("<blockquote><b>{}</b>\n{}</blockquote>", thinking_header, escape_html(&current)));
        }
    }
    
    let answer = answer.trim();
    if !answer.is_empty() {
        let answer_chunks = markdown_to_telegram_html_chunks(answer);
        chunks.extend(answer_chunks);
    } else if !thinking.is_empty() {
        if let Some(last) = chunks.last_mut() {
            last.push_str("\n\n✅ Done (Thinking only)");
        }
    }
    
    if chunks.is_empty() {
        chunks.push(String::new());
    }
    
    chunks
}

pub async fn create_session(client: &Client, server_url: &str, directory: Option<&str>) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let mut req = client.post(format!("{}/session", server_url));
    let mut title = "Telegram Bot Session".to_string();

    if let Some(dir) = directory {
        req = req.query(&[("directory", dir)]);
        req = req.header("x-opencode-directory", dir);
        
        let dir_path = std::path::Path::new(dir);
        if let Some(name) = dir_path.file_name().and_then(|n| n.to_str()) {
            title = format!("Session: {}", name);
        } else {
            title = format!("Session in {}", dir);
        }
    }
    
    let res = req.json(&json!({"title": title})).send().await?;
        
    if !res.status().is_success() {
        return Err(res.text().await?.into());
    }
    
    let data: Value = res.json().await?;
    Ok(data["id"].as_str().unwrap_or("").to_string())
}

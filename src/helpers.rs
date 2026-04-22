use reqwest::Client;
use serde_json::{json, Value};
use teloxide::types::{KeyboardButton, KeyboardMarkup};
use crate::markdown::markdown_to_telegram_html_chunks;

pub fn main_menu_keyboard() -> KeyboardMarkup {
    KeyboardMarkup::new(vec![
        vec![
            KeyboardButton::new("🔄 New Session"),
            KeyboardButton::new("📁 Set Project"),
        ],
        vec![
            KeyboardButton::new("🤖 Change Model"),
            KeyboardButton::new("📜 List Sessions"),
        ],
        vec![
            KeyboardButton::new("❓ Help"),
        ],
    ])
    .resize_keyboard()
}

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
    if let Some(dir) = directory {
        req = req.query(&[("directory", dir)]);
        req = req.header("x-opencode-directory", dir);
    }
    
    let res = req.json(&json!({"title": "Telegram Bot Session"})).send().await?;
        
    if !res.status().is_success() {
        return Err(res.text().await?.into());
    }
    
    let data: Value = res.json().await?;
    Ok(data["id"].as_str().unwrap_or("").to_string())
}

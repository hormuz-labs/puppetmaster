use reqwest::Client;
use serde_json::{json, Value};
use teloxide::types::{KeyboardButton, KeyboardMarkup};

pub fn main_menu_keyboard() -> KeyboardMarkup {
    KeyboardMarkup::new(vec![
        vec![
            KeyboardButton::new("🔄 New Session"),
            KeyboardButton::new("📁 Set Project"),
        ],
        vec![
            KeyboardButton::new("🤖 Change Model"),
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

use crate::markdown::markdown_to_telegram_html;

pub fn render_html_chunks(thinking: &str, answer: &str, thinking_header: &str) -> Vec<String> {
    let mut chunks = Vec::new();
    let limit = 3900;
    
    // We'll build the final telegram HTML then chunk it.
    // While chunking HTML is hard, Telegram is somewhat forgiving,
    // but splitting in the middle of a tag is bad.
    
    // Actually, maybe we can just build strings and split?
    let mut full_html = String::new();
    
    if !thinking.trim().is_empty() {
        let escaped = escape_html(thinking.trim());
        full_html.push_str(&format!("<blockquote><b>{}</b>\n{}\n</blockquote>\n\n", thinking_header, escaped));
    }
    
    if !answer.trim().is_empty() {
        let answer_html = markdown_to_telegram_html(answer);
        full_html.push_str(&answer_html);
    } else if thinking.trim().is_empty() {
        // Nothing at all
    } else {
        full_html.push_str("✅ Done (Thinking only)");
    }
    
    if full_html.is_empty() {
        return vec![String::new()];
    }

    // A simple chunker that tries to split by newlines without breaking HTML tags
    let mut current_chunk = String::new();
    
    for line in full_html.split_inclusive('\n') {
        if current_chunk.len() + line.len() > limit {
            chunks.push(current_chunk.clone());
            current_chunk.clear();
        }
        current_chunk.push_str(line);
    }
    
    if !current_chunk.is_empty() {
        chunks.push(current_chunk);
    }

    chunks
}

pub async fn create_session(client: &Client, server_url: &str, directory: Option<&str>) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let mut req = client.post(format!("{}/session", server_url));
    if let Some(dir) = directory {
        req = req.query(&[("directory", dir)]);
    }
    
    let res = req.json(&json!({"title": "Telegram Bot Session"})).send().await?;
        
    if !res.status().is_success() {
        return Err(res.text().await?.into());
    }
    
    let data: Value = res.json().await?;
    Ok(data["id"].as_str().unwrap_or("").to_string())
}

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

pub fn render_html_chunks(thinking: &str, answer: &str, thinking_header: &str) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut current_chunk = String::new();
    let limit = 3900;
    
    // 1. Process Thinking Text
    if !thinking.trim().is_empty() {
        let open_tag = format!("<blockquote expandable><b>{}</b>\n", thinking_header);
        let close_tag = "</blockquote>\n\n";
        
        current_chunk.push_str(&open_tag);
        
        for line in thinking.split('\n') {
            let escaped = escape_html(line);
            
            if current_chunk.len() + escaped.len() + close_tag.len() > limit {
                current_chunk.push_str("</blockquote>");
                chunks.push(current_chunk.clone());
                current_chunk.clear();
                current_chunk.push_str(&open_tag);
            }
            
            // If a single line is still too long, force split it
            if escaped.len() > limit {
                let mut i = 0;
                while i < escaped.len() {
                    let end = std::cmp::min(i + limit - close_tag.len() - open_tag.len() - 10, escaped.len());
                    current_chunk.push_str(&escaped[i..end]);
                    if end < escaped.len() {
                        current_chunk.push_str("</blockquote>");
                        chunks.push(current_chunk.clone());
                        current_chunk.clear();
                        current_chunk.push_str(&open_tag);
                    }
                    i = end;
                }
                current_chunk.push('\n');
                continue;
            }
            
            current_chunk.push_str(&escaped);
            current_chunk.push('\n');
        }
        current_chunk.push_str(close_tag);
    }
    
    // 2. Process Answer Text
    if !answer.trim().is_empty() {
        let mut in_code_block = false;
        let mut current_lang = String::new();
        
        for line in answer.split('\n') {
            if line.starts_with("```") {
                in_code_block = !in_code_block;
                if in_code_block {
                    current_lang = line.trim_start_matches("```").trim().to_string();
                    let tag = if !current_lang.is_empty() {
                        format!("<pre><code class=\"language-{}\">", current_lang)
                    } else {
                        "<pre><code>".to_string()
                    };
                    
                    if current_chunk.len() + tag.len() > limit {
                        chunks.push(current_chunk.clone());
                        current_chunk.clear();
                    }
                    current_chunk.push_str(&tag);
                } else {
                    let tag = "</code></pre>\n";
                    if current_chunk.len() + tag.len() > limit {
                        current_chunk.push_str("</code></pre>");
                        chunks.push(current_chunk.clone());
                        current_chunk.clear();
                    } else {
                        current_chunk.push_str(tag);
                    }
                }
                continue;
            }
            
            let mut processed_line = String::new();
            let mut in_inline_code = false;
            let mut chars = line.chars().peekable();
            
            while let Some(c) = chars.next() {
                if c == '`' && !in_code_block {
                    in_inline_code = !in_inline_code;
                    if in_inline_code {
                        processed_line.push_str("<code>");
                    } else {
                        processed_line.push_str("</code>");
                    }
                } else if c == '<' {
                    processed_line.push_str("&lt;");
                } else if c == '>' {
                    processed_line.push_str("&gt;");
                } else if c == '&' {
                    processed_line.push_str("&amp;");
                } else {
                    processed_line.push(c);
                }
            }
            processed_line.push('\n');
            
            if current_chunk.len() + processed_line.len() > limit {
                if in_code_block {
                    current_chunk.push_str("</code></pre>");
                }
                chunks.push(current_chunk.clone());
                current_chunk.clear();
                if in_code_block {
                    let tag = if !current_lang.is_empty() {
                        format!("<pre><code class=\"language-{}\">", current_lang)
                    } else {
                        "<pre><code>".to_string()
                    };
                    current_chunk.push_str(&tag);
                }
            }
            current_chunk.push_str(&processed_line);
        }
        
        if in_code_block {
            current_chunk.push_str("</code></pre>");
        }
    } else if answer.trim().is_empty() && thinking.trim().is_empty() {
        // Nothing at all
    } else if answer.trim().is_empty() && !thinking.trim().is_empty() {
        current_chunk.push_str("✅ Done (Thinking only)");
    }
    
    if !current_chunk.trim().is_empty() {
        chunks.push(current_chunk);
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
    }
    
    let res = req.json(&json!({"title": "Telegram Bot Session"})).send().await?;
        
    if !res.status().is_success() {
        return Err(res.text().await?.into());
    }
    
    let data: Value = res.json().await?;
    Ok(data["id"].as_str().unwrap_or("").to_string())
}

use anyhow::{Context, Result};
use clap::Parser;
use reqwest::multipart;
use std::env;
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about = "Send messages and files to Telegram")]
struct Cli {
    /// The message or caption to send
    message: String,

    /// Optional path to a file (image, video, etc.) to send
    file: Option<PathBuf>,
}

fn get_endpoint_and_param(mime: &mime_guess::Mime) -> (&'static str, &'static str) {
    match mime.type_() {
        mime_guess::mime::IMAGE => ("sendPhoto", "photo"),
        mime_guess::mime::VIDEO => ("sendVideo", "video"),
        mime_guess::mime::AUDIO => ("sendAudio", "audio"),
        _ => ("sendDocument", "document"),
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Try to load environment variables
    // Priority 1: ~/.puppetmaster/.env (Global config from onboarding)
    if let Some(home) = dirs::home_dir() {
        let global_config = home.join(".puppetmaster").join(".env");
        if global_config.exists() {
            // Use dotenv_override so the file overrides any currently set vars (like in our shell session)
            let _ = dotenvy::from_path_override(&global_config);
        }
    }

    // Priority 2: Local .env (Project specific overrides)
    let _ = dotenvy::dotenv_override();
    
    // Priority 3: Fallback check parent directories
    if env::var("TELOXIDE_TOKEN").is_err() {
        let _ = dotenvy::from_path_override("../../.env");
    }

    let args = Cli::parse();

    // 2. Resolve credentials
    let token = env::var("TELEGRAM_BOT_TOKEN")
        .or_else(|_| env::var("TELOXIDE_TOKEN"))
        .context("Missing TELEGRAM_BOT_TOKEN or TELOXIDE_TOKEN")?;

    let mut chat_id = env::var("TELEGRAM_CHAT_ID").ok();
    
    // Smart fallback for puppetmaster project
    if chat_id.is_none() {
        if let Ok(allowed) = env::var("ALLOWED_USERS") {
            if allowed != "*" {
                chat_id = allowed.split(',').next().map(|s| s.trim().to_string());
            }
        }
    }
    
    let chat_id = chat_id.context("Missing TELEGRAM_CHAT_ID or ALLOWED_USERS")?;

    // 3. Send to Telegram
    let client = reqwest::Client::new();
    
    if let Some(file_path) = args.file {
        if !file_path.exists() {
            anyhow::bail!("File not found: {:?}", file_path);
        }

        let mime = mime_guess::from_path(&file_path).first_or_octet_stream();
        let (endpoint, param) = get_endpoint_and_param(&mime);

        let file_name = file_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("file")
            .to_string();

        let file_bytes = tokio::fs::read(&file_path).await?;
        let file_part = multipart::Part::bytes(file_bytes)
            .file_name(file_name)
            .mime_str(mime.as_ref())?;

        let form = multipart::Form::new()
            .text("chat_id", chat_id)
            .text("caption", args.message)
            .part(param, file_part);

        let url = format!("https://api.telegram.org/bot{}/{}", token, endpoint);
        let res = client.post(url).multipart(form).send().await?;

        if !res.status().is_success() {
            anyhow::bail!("Telegram API Error: {}", res.text().await?);
        }
        println!("✅ File sent successfully!");
    } else {
        let url = format!("https://api.telegram.org/bot{}/sendMessage", token);
        let res = client.post(url)
            .json(&serde_json::json!({
                "chat_id": chat_id,
                "text": args.message,
                "parse_mode": "Markdown"
            }))
            .send()
            .await?;

        if !res.status().is_success() {
            anyhow::bail!("Telegram API Error: {}", res.text().await?);
        }
        println!("✅ Message sent successfully!");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_mime_type_resolution() {
        let cases = vec![
            ("image/jpeg", "sendPhoto", "photo"),
            ("image/png", "sendPhoto", "photo"),
            ("video/mp4", "sendVideo", "video"),
            ("audio/mpeg", "sendAudio", "audio"),
            ("application/pdf", "sendDocument", "document"),
            ("text/plain", "sendDocument", "document"),
            ("application/octet-stream", "sendDocument", "document"),
        ];

        for (mime_str, expected_endpoint, expected_param) in cases {
            let mime = mime_guess::Mime::from_str(mime_str).unwrap();
            let (endpoint, param) = get_endpoint_and_param(&mime);
            assert_eq!(endpoint, expected_endpoint, "Failed for MIME: {}", mime_str);
            assert_eq!(param, expected_param, "Failed for MIME: {}", mime_str);
        }
    }
}

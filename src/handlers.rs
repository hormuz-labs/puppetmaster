use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use futures_util::StreamExt;
use reqwest::Client;
use reqwest_eventsource::{Event, EventSource};
use serde_json::{json, Value};
use teloxide::{
    dispatching::dialogue::InMemStorage,
    prelude::*,
    types::{KeyboardButton, KeyboardRemove, KeyboardMarkup, ParseMode, ChatAction},
    utils::command::BotCommands,
};
use tracing::{error, info};
use std::env;
use base64::{Engine as _, engine::general_purpose::STANDARD};
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use teloxide::net::Download;

use crate::state::{State, Command};
use crate::helpers::{main_menu_keyboard, render_html_chunks, create_session};

pub type MyDialogue = Dialogue<State, InMemStorage<State>>;
pub type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

pub async fn help_command(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, Command::descriptions().to_string()).await?;
    Ok(())
}

pub async fn start_command(bot: Bot, msg: Message, dialogue: MyDialogue, client: Client, server_url: Arc<String>) -> HandlerResult {
    let chat_id = msg.chat.id;
    bot.send_message(chat_id, "Welcome to OpenCode! Creating your first session...")
        .reply_markup(main_menu_keyboard())
        .await?;
    
    match create_session(&client, &server_url, None).await {
        Ok(session_id) => {
            bot.send_message(chat_id, format!("✅ Created new session: `{}`", session_id)).await?;
            dialogue.update(State::ActiveSession { 
                session_id, 
                directory: None, 
                model: None 
            }).await?;
        }
        Err(e) => {
            bot.send_message(chat_id, format!("❌ Failed to create session: {}", e)).await?;
        }
    }
    
    Ok(())
}

pub async fn session_command_text(bot: Bot, msg: Message, dialogue: MyDialogue, client: Client, server_url: Arc<String>) -> HandlerResult {
    session_command(bot, msg, dialogue, client, server_url).await
}

pub async fn project_command_text(bot: Bot, msg: Message, dialogue: MyDialogue, client: Client, server_url: Arc<String>) -> HandlerResult {
    project_command(bot, msg, dialogue, client, server_url).await
}

pub async fn model_command_text(bot: Bot, msg: Message, dialogue: MyDialogue, client: Client, server_url: Arc<String>) -> HandlerResult {
    model_command(bot, msg, dialogue, client, server_url).await
}

pub async fn session_command(bot: Bot, msg: Message, dialogue: MyDialogue, client: Client, server_url: Arc<String>) -> HandlerResult {
    let chat_id = msg.chat.id;
    let state = dialogue.get().await?.unwrap_or_default();
    
    let current_dir = match state {
        State::ActiveSession { directory, .. } => directory,
        State::AwaitingModel { directory, .. } => directory,
        _ => None,
    };
    
    bot.send_message(chat_id, "Creating a new session...").await?;
    
    match create_session(&client, &server_url, current_dir.as_deref()).await {
        Ok(session_id) => {
            bot.send_message(chat_id, format!("✅ Created new session: `{}`", session_id)).await?;
            dialogue.update(State::ActiveSession { 
                session_id, 
                directory: current_dir, 
                model: None 
            }).await?;
        }
        Err(e) => {
            bot.send_message(chat_id, format!("❌ Failed to create session: {}", e)).await?;
        }
    }
    
    Ok(())
}

pub async fn project_command(bot: Bot, msg: Message, dialogue: MyDialogue, client: Client, server_url: Arc<String>) -> HandlerResult {
    let state = dialogue.get().await?.unwrap_or_default();
    
    let (prev_session_id, prev_directory, model) = match state {
        State::ActiveSession { session_id, directory, model } => (Some(session_id), directory, model),
        State::AwaitingProjectDir { prev_session_id, prev_directory, model } => (prev_session_id, prev_directory, model),
        State::AwaitingModel { session_id, directory } => (session_id, directory, None),
        _ => (None, None, None),
    };

    let mut keyboard_rows = vec![];
    
    if let Ok(res) = client.get(format!("{}/project", server_url)).send().await {
        if let Ok(projects) = res.json::<Vec<Value>>().await {
            let mut sorted_projects = projects;
            sorted_projects.sort_by(|a, b| {
                let time_a = a["time"]["updated"].as_i64().unwrap_or(0);
                let time_b = b["time"]["updated"].as_i64().unwrap_or(0);
                time_b.cmp(&time_a)
            });
            
            for proj in sorted_projects.iter().take(10) {
                if let Some(worktree) = proj["worktree"].as_str() {
                    if worktree != "/" {
                        keyboard_rows.push(vec![KeyboardButton::new(worktree.to_string())]);
                    }
                }
            }
        }
    }
    
    keyboard_rows.push(vec![
        KeyboardButton::new("🔙 Cancel / Go Back"),
        KeyboardButton::new("⌨️ Type manually"),
    ]);

    let keyboard = KeyboardMarkup::new(keyboard_rows)
        .resize_keyboard()
        .one_time_keyboard();

    bot.send_message(msg.chat.id, "Please select a recent project directory, or type an absolute path:")
        .reply_markup(keyboard)
        .await?;
        
    dialogue.update(State::AwaitingProjectDir { prev_session_id, prev_directory, model }).await?;
    
    Ok(())
}

pub async fn receive_project_dir(
    bot: Bot, 
    msg: Message, 
    dialogue: MyDialogue, 
    (prev_session_id, prev_directory, model): (Option<String>, Option<String>, Option<String>), 
    client: Client, 
    server_url: Arc<String>
) -> HandlerResult {
    let dir = msg.text().unwrap_or("").to_string();
    
    if dir.contains("Cancel / Go Back") {
        let bot_msg = bot.send_message(msg.chat.id, "Going back...")
            .reply_markup(main_menu_keyboard())
            .await?;
        let _ = bot.delete_message(msg.chat.id, bot_msg.id).await;
        
        if let Some(sid) = prev_session_id {
            dialogue.update(State::ActiveSession { session_id: sid, directory: prev_directory, model }).await?;
        } else {
            dialogue.update(State::Start).await?;
        }
        return Ok(());
    }
    
    if dir.contains("Type manually") {
        bot.send_message(msg.chat.id, "Okay, please type the absolute path manually.")
            .reply_markup(KeyboardRemove::new())
            .await?;
        return Ok(());
    }
    
    bot.send_message(msg.chat.id, format!("Setting project directory to `{}` and creating a new session...", dir))
        .reply_markup(main_menu_keyboard())
        .await?;
        
    match create_session(&client, &server_url, Some(&dir)).await {
        Ok(session_id) => {
            bot.send_message(msg.chat.id, format!("✅ Project set! Created new session: `{}`", session_id)).await?;
            dialogue.update(State::ActiveSession { 
                session_id, 
                directory: Some(dir), 
                model 
            }).await?;
        }
        Err(e) => {
            bot.send_message(msg.chat.id, format!("❌ Failed to create session with that directory: {}", e)).await?;
            if let Some(sid) = prev_session_id {
                dialogue.update(State::ActiveSession { session_id: sid, directory: None, model }).await?;
            } else {
                dialogue.update(State::Start).await?;
            }
        }
    }
    
    Ok(())
}

pub async fn model_command(bot: Bot, msg: Message, dialogue: MyDialogue, client: Client, server_url: Arc<String>) -> HandlerResult {
    let state = dialogue.get().await?.unwrap_or_default();
    
    let (session_id, directory) = match state {
        State::ActiveSession { session_id, directory, .. } => (Some(session_id), directory),
        State::AwaitingProjectDir { prev_session_id, .. } => (prev_session_id, None),
        State::AwaitingModel { session_id, directory } => (session_id, directory),
        _ => (None, None),
    };

    let mut keyboard_rows = vec![];
    let mut current_row = vec![];
    
    if let Ok(res) = client.get(format!("{}/provider", server_url)).send().await {
        if let Ok(data) = res.json::<Value>().await {
            if let (Some(connected), Some(all)) = (data["connected"].as_array(), data["all"].as_array()) {
                let connected_ids: Vec<&str> = connected.iter().filter_map(|v| v.as_str()).collect();
                
                for provider in all {
                    if let Some(id) = provider["id"].as_str() {
                        if connected_ids.contains(&id) {
                            if let Some(models) = provider["models"].as_object() {
                                for (_, model_info) in models {
                                    if let Some(model_id) = model_info["id"].as_str() {
                                        let full_id = format!("{}/{}", id, model_id);
                                        current_row.push(KeyboardButton::new(full_id));
                                        
                                        if current_row.len() >= 2 {
                                            keyboard_rows.push(current_row.clone());
                                            current_row.clear();
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    if !current_row.is_empty() {
        keyboard_rows.push(current_row);
    }
    
    keyboard_rows.push(vec![KeyboardButton::new("🔙 Cancel / Go Back")]);

    let keyboard = KeyboardMarkup::new(keyboard_rows)
        .resize_keyboard()
        .one_time_keyboard();

    bot.send_message(msg.chat.id, "Please select the model you want to use:")
        .reply_markup(keyboard)
        .await?;
        
    dialogue.update(State::AwaitingModel { session_id, directory }).await?;
    
    Ok(())
}

pub async fn receive_model(
    bot: Bot, 
    msg: Message, 
    dialogue: MyDialogue, 
    (session_id, directory): (Option<String>, Option<String>)
) -> HandlerResult {
    let model = msg.text().unwrap_or("").to_string();
    
    if model.contains("Cancel / Go Back") {
        let bot_msg = bot.send_message(msg.chat.id, "Going back...")
            .reply_markup(main_menu_keyboard())
            .await?;
        let _ = bot.delete_message(msg.chat.id, bot_msg.id).await;
            
        if let Some(sid) = session_id {
            dialogue.update(State::ActiveSession { session_id: sid, directory, model: None }).await?;
        } else {
            dialogue.update(State::Start).await?;
        }
        return Ok(());
    }
    
    if let Some(sid) = session_id {
        bot.send_message(msg.chat.id, format!("✅ Active model set to: `{}`", model))
            .reply_markup(main_menu_keyboard())
            .await?;
            
        dialogue.update(State::ActiveSession { 
            session_id: sid, 
            directory, 
            model: Some(model) 
        }).await?;
    } else {
        bot.send_message(msg.chat.id, "✅ Model set. Please use /start or /session to create a session first.")
            .reply_markup(main_menu_keyboard())
            .await?;
            
        dialogue.update(State::Start).await?; 
    }
    
    Ok(())
}

pub async fn handle_no_session(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, "Please /start the bot or create a /session first.").await?;
    Ok(())
}

pub async fn handle_prompt(
    bot: Bot, 
    msg: Message, 
    (session_id, _directory, model): (String, Option<String>, Option<String>), 
    client: Client, 
    server_url: Arc<String>
) -> HandlerResult {
    let chat_id = msg.chat.id;
    let mut text = String::new();
    
    if let Some(t) = msg.text() {
        text = t.to_string();
    } else if let Some(voice) = msg.voice() {
        let bot_msg = bot.send_message(chat_id, "🎙 Processing voice message...").await?;
        let google_api_key = env::var("GOOGLE_SPEECH_API_KEY").unwrap_or_else(|_| env::var("GOOGLE_API_KEY").unwrap_or_default());
        if google_api_key.is_empty() {
            let _ = bot.edit_message_text(chat_id, bot_msg.id, "❌ GOOGLE_SPEECH_API_KEY not set in .env. Voice messages are not supported.").await;
            return Ok(());
        }

        let file = match bot.get_file(voice.file.id.clone()).await {
            Ok(f) => f,
            Err(e) => {
                let _ = bot.edit_message_text(chat_id, bot_msg.id, format!("❌ Failed to get voice file: {}", e)).await;
                return Ok(());
            }
        };
        
        let temp_path = format!("/tmp/voice_{}.ogg", voice.file.id);
        let mut temp_file = match tokio::fs::File::create(&temp_path).await {
            Ok(f) => f,
            Err(e) => {
                let _ = bot.edit_message_text(chat_id, bot_msg.id, format!("❌ Failed to create temp file: {}", e)).await;
                return Ok(());
            }
        };

        if let Err(e) = bot.download_file(&file.path, &mut temp_file).await {
            let _ = bot.edit_message_text(chat_id, bot_msg.id, format!("❌ Failed to download voice: {}", e)).await;
            return Ok(());
        }

        let mut buffer = Vec::new();
        let mut temp_file = tokio::fs::File::open(&temp_path).await?;
        temp_file.read_to_end(&mut buffer).await?;
        let _ = tokio::fs::remove_file(&temp_path).await;

        let base64_audio = STANDARD.encode(&buffer);
        let payload = json!({
            "config": {
                "encoding": "OGG_OPUS",
                "sampleRateHertz": 48000,
                "languageCode": "en-US"
            },
            "audio": {
                "content": base64_audio
            }
        });

        let res = client.post(format!("https://speech.googleapis.com/v1/speech:recognize?key={}", google_api_key))
            .json(&payload)
            .send()
            .await;

        match res {
            Ok(response) if response.status().is_success() => {
                let json_data: Value = response.json().await?;
                if let Some(results) = json_data["results"].as_array() {
                    if let Some(first) = results.first() {
                        if let Some(alternatives) = first["alternatives"].as_array() {
                            if let Some(first_alt) = alternatives.first() {
                                if let Some(transcript) = first_alt["transcript"].as_str() {
                                    text = transcript.to_string();
                                    let _ = bot.edit_message_text(chat_id, bot_msg.id, format!("🎙 Transcribed:\n\n_{}_", text)).parse_mode(ParseMode::Markdown).await;
                                }
                            }
                        }
                    }
                }
                
                if text.is_empty() {
                    let _ = bot.edit_message_text(chat_id, bot_msg.id, "❌ Could not transcribe the voice message.").await;
                    return Ok(());
                }
            }
            Ok(response) => {
                let error_text = response.text().await.unwrap_or_default();
                error!("Google Speech API error: {}", error_text);
                let _ = bot.edit_message_text(chat_id, bot_msg.id, "❌ Google Speech API returned an error.").await;
                return Ok(());
            }
            Err(e) => {
                error!("Google Speech network error: {}", e);
                let _ = bot.edit_message_text(chat_id, bot_msg.id, "❌ Network error reaching Google Speech API.").await;
                return Ok(());
            }
        }
    } else {
        return Ok(());
    }
    
    let bot_msg = match bot.send_message(chat_id, "⏳ Thinking...").await {
        Ok(m) => m,
        Err(e) => {
            error!("Failed to send thinking message: {}", e);
            return Ok(());
        }
    };
    
    let mut sse_req = client.get(format!("{}/event?sessionID={}", server_url, session_id));
    if let Some(ref dir) = _directory {
        sse_req = sse_req.header("x-opencode-directory", dir);
    }
    let sse_req = sse_req.try_clone().unwrap();
    
    let mut es = match EventSource::new(sse_req) {
        Ok(es) => es,
        Err(e) => {
            error!("Failed to create EventSource: {}", e);
            let _ = bot.edit_message_text(chat_id, bot_msg.id, format!("❌ Connection Error: {}", e)).await;
            return Ok(());
        }
    };

    let mut payload = json!({
        "parts": [{"type": "text", "text": text}]
    });
    
    if let Some(ref m) = model {
        if let Some((provider, model_id)) = m.split_once('/') {
            payload["model"] = json!({
                "providerID": provider,
                "modelID": model_id
            });
        }
    }

    let mut req = client.post(format!("{}/session/{}/prompt_async", server_url, session_id));
    if let Some(ref dir) = _directory {
        req = req.header("x-opencode-directory", dir);
    }
    let prompt_res = req
        .json(&payload)
        .send()
        .await;
        
    match prompt_res {
        Ok(res) if res.status().is_success() => {},
        Ok(res) => {
            let err = res.text().await.unwrap_or_default();
            error!("Failed to send prompt: {}", err);
            let _ = bot.edit_message_text(chat_id, bot_msg.id, format!("❌ Error sending prompt: {}", err)).await;
            return Ok(());
        }
        Err(e) => {
            error!("Failed to send prompt: {}", e);
            let _ = bot.edit_message_text(chat_id, bot_msg.id, format!("❌ Network Error: {}", e)).await;
            return Ok(());
        }
    }
    
    let mut thinking_text = String::new();
    let mut answer_text = String::new();
    let mut part_types = HashMap::<String, String>::new();
    let mut sent_messages = vec![bot_msg.id];
    let mut last_edit = Instant::now();
    let mut last_typing = Instant::now();
    let _ = bot.send_chat_action(chat_id, ChatAction::Typing).await;
    
    loop {
        if last_typing.elapsed() > Duration::from_secs(4) {
            let _ = bot.send_chat_action(chat_id, ChatAction::Typing).await;
            last_typing = Instant::now();
        }

        let event_res = match tokio::time::timeout(Duration::from_secs(1), es.next()).await {
            Ok(Some(ev)) => ev,
            Ok(None) => break,
            Err(_) => continue,
        };
        if last_typing.elapsed() > Duration::from_secs(4) {
            let _ = bot.send_chat_action(chat_id, ChatAction::Typing).await;
            last_typing = Instant::now();
        }
        match event_res {
            Ok(Event::Open) => {
                info!("SSE Connection Opened!");
            }
            Ok(Event::Message(message)) => {
                if let Ok(data) = serde_json::from_str::<Value>(&message.data) {
                    let event_type = data["type"].as_str().unwrap_or("");
                    let properties = &data["properties"];
                    
                    match event_type {
                        "message.part.updated" => {
                            if let (Some(pid), Some(ptype)) = (properties["part"]["id"].as_str(), properties["part"]["type"].as_str()) {
                                part_types.insert(pid.to_string(), ptype.to_string());
                            }
                        }
                        "message.part.delta" => {
                            let part_id = properties["partID"].as_str().unwrap_or("");
                            let delta = properties["delta"].as_str().unwrap_or("");
                            let field = properties["field"].as_str().unwrap_or("");

                            if field == "text" || field == "reasoning" {
                                let part_type = part_types.get(part_id).map(|s| s.as_str()).unwrap_or("text");
                                if part_type == "reasoning" || field == "reasoning" {
                                    thinking_text.push_str(delta);
                                } else {
                                    answer_text.push_str(delta);
                                }
                                
                                if last_edit.elapsed() > Duration::from_secs(1) {
                                    let frame = match (last_edit.elapsed().as_millis() / 500) % 3 {
                                        0 => "⏳ Thinking.",
                                        1 => "⏳ Thinking..",
                                        _ => "⏳ Thinking...",
                                    };
                                    let chunks = render_html_chunks(&thinking_text, &answer_text, frame);

                                    if let Some(last_chunk) = chunks.last() {
                                        if !last_chunk.trim().is_empty() {
                                            while sent_messages.len() < chunks.len() {
                                                if let Ok(new_msg) = bot.send_message(chat_id, "⏳...").await {
                                                    sent_messages.push(new_msg.id);
                                                } else {
                                                    break;
                                                }
                                            }
                                            
                                            let last_chunk_index = chunks.len() - 1;
                                            let last_msg_id = sent_messages[last_chunk_index];
                                            let _ = bot.edit_message_text(chat_id, last_msg_id, last_chunk)
                                                .parse_mode(ParseMode::Html)
                                                .await;
                                        }
                                    }
                                    last_edit = Instant::now();
                                }
                            }
                        }
                        "session.idle" => {
                            info!("Session became idle, stream complete.");
                            break;
                        }
                        "session.error" => {
                            let err_msg = properties.to_string();
                            error!("Session error: {}", err_msg);
                            answer_text.push_str(&format!("\n\n❌ Session Error: {}", err_msg));
                            break;
                        }
                        _ => {}
                    }
                }
            }
            Err(err) => {
                error!("SSE Stream Error: {}", err);
                answer_text.push_str("\n\n⚠️ Stream interrupted.");
                break;
            }
        }
    }
    
    es.close();
    
    let chunks = render_html_chunks(&thinking_text, &answer_text, "💭 Thinking completed");
    
    if chunks.len() == 1 && chunks[0].trim().is_empty() {
        let _ = bot.edit_message_text(chat_id, sent_messages[0], "✅ Done (No text output)").await;
        return Ok(());
    }
    
    while sent_messages.len() < chunks.len() {
        if let Ok(new_msg) = bot.send_message(chat_id, "⏳...").await {
            sent_messages.push(new_msg.id);
        } else {
            break;
        }
    }
    
    for (i, chunk) in chunks.iter().enumerate() {
        if i < sent_messages.len() {
            let res = bot.edit_message_text(chat_id, sent_messages[i], chunk)
                .parse_mode(ParseMode::Html)
                .await;
                
            if res.is_err() {
                let _ = bot.edit_message_text(chat_id, sent_messages[i], chunk).await;
            }
        }
    }
    
    Ok(())
}

pub async fn abort_command(bot: Bot, msg: Message, dialogue: MyDialogue, client: Client, server_url: Arc<String>) -> HandlerResult {
    let state = dialogue.get().await?.unwrap_or_default();
    
    if let State::ActiveSession { session_id, directory, .. } = state {
        let mut req = client.post(format!("{}/session/{}/abort", server_url, session_id));
        if let Some(dir) = directory {
            req = req.header("x-opencode-directory", dir);
        }
        match req.send().await {
            Ok(_) => {
                bot.send_message(msg.chat.id, "🛑 Sent abort signal to the active session.").await?;
            }
            Err(e) => {
                bot.send_message(msg.chat.id, format!("❌ Failed to abort: {}", e)).await?;
            }
        }
    } else {
        bot.send_message(msg.chat.id, "No active session to abort.").await?;
    }
    
    Ok(())
}

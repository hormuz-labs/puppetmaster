use std::env;
use std::sync::Arc;
use std::time::{Duration, Instant};

use futures_util::StreamExt;
use reqwest::Client;
use reqwest_eventsource::{Event, EventSource};
use serde_json::{json, Value};
use teloxide::{
    dispatching::{dialogue, dialogue::InMemStorage, UpdateHandler},
    prelude::*,
    types::{KeyboardButton, KeyboardRemove, KeyboardMarkup},
    utils::command::BotCommands,
};
use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;

type MyDialogue = Dialogue<State, InMemStorage<State>>;
type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

#[derive(Clone, Default, Debug)]
pub enum State {
    #[default]
    Start,
    ActiveSession {
        session_id: String,
        directory: Option<String>,
        model: Option<String>,
    },
    AwaitingProjectDir {
        prev_session_id: Option<String>,
        model: Option<String>,
    },
    AwaitingModel {
        session_id: Option<String>,
        directory: Option<String>,
    },
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "These commands are supported:")]
enum Command {
    #[command(description = "Display this help text.")]
    Help,
    #[command(description = "Start the bot and initialize a session.")]
    Start,
    #[command(description = "Create a new session.")]
    Session,
    #[command(description = "Set the active project directory.")]
    Project,
    #[command(description = "Set the AI model (e.g., google/gemini-3-pro-preview).")]
    Model,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default subscriber failed");

    // Load environment variables
    dotenvy::dotenv().ok();

    let bot_token = env::var("TELOXIDE_TOKEN").expect("TELOXIDE_TOKEN must be set");
    let server_url = env::var("OPENCODE_SERVER_URL").unwrap_or_else(|_| "http://127.0.0.1:4096".to_string());
    
    info!("Starting OpenCode Telegram Bot (Stage 3)...");
    
    let bot = Bot::new(bot_token);
    
    // Set up the Telegram bot menu
    let commands = Command::bot_commands();
    if let Err(e) = bot.set_my_commands(commands).await {
        error!("Failed to set bot commands: {}", e);
    }
    
    let http_client = Client::new();
    let server_url = Arc::new(server_url);

    Dispatcher::builder(bot, schema())
        .dependencies(dptree::deps![
            InMemStorage::<State>::new(),
            http_client,
            server_url
        ])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    Ok(())
}

fn schema() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync + 'static>> {
    use dptree::case;

    let command_handler = teloxide::filter_command::<Command, _>()
        .branch(case![Command::Start].endpoint(start_command))
        .branch(case![Command::Help].endpoint(help_command))
        .branch(case![Command::Session].endpoint(session_command))
        .branch(case![Command::Project].endpoint(project_command))
        .branch(case![Command::Model].endpoint(model_command));

    let message_handler = Update::filter_message()
        .branch(command_handler)
        .branch(case![State::AwaitingProjectDir { prev_session_id, model }].endpoint(receive_project_dir))
        .branch(case![State::AwaitingModel { session_id, directory }].endpoint(receive_model))
        .branch(case![State::ActiveSession { session_id, directory, model }].endpoint(handle_prompt))
        .branch(dptree::endpoint(handle_no_session));

    dialogue::enter::<Update, InMemStorage<State>, State, _>()
        .branch(message_handler)
}

// --- Helpers ---

async fn create_session(client: &Client, server_url: &str, directory: Option<&str>) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
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

// --- Handlers ---

async fn help_command(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, Command::descriptions().to_string()).await?;
    Ok(())
}

async fn start_command(bot: Bot, msg: Message, dialogue: MyDialogue, client: Client, server_url: Arc<String>) -> HandlerResult {
    let chat_id = msg.chat.id;
    bot.send_message(chat_id, "Welcome to OpenCode! Creating your first session...").await?;
    
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

async fn session_command(bot: Bot, msg: Message, dialogue: MyDialogue, client: Client, server_url: Arc<String>) -> HandlerResult {
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
            // Preserve directory and model if possible, but we don't have model here easily, let's just reset model for now or we can extract it.
            // Simplified:
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

async fn project_command(bot: Bot, msg: Message, dialogue: MyDialogue) -> HandlerResult {
    let state = dialogue.get().await?.unwrap_or_default();
    
    let (prev_session_id, model) = match state {
        State::ActiveSession { session_id, model, .. } => (Some(session_id), model),
        State::AwaitingProjectDir { prev_session_id, model } => (prev_session_id, model),
        State::AwaitingModel { session_id, .. } => (session_id, None),
        _ => (None, None),
    };

    bot.send_message(msg.chat.id, "Please send the absolute path to your project directory:").await?;
    dialogue.update(State::AwaitingProjectDir { prev_session_id, model }).await?;
    
    Ok(())
}

async fn receive_project_dir(
    bot: Bot, 
    msg: Message, 
    dialogue: MyDialogue, 
    (prev_session_id, model): (Option<String>, Option<String>), 
    client: Client, 
    server_url: Arc<String>
) -> HandlerResult {
    let dir = msg.text().unwrap_or("").to_string();
    
    bot.send_message(msg.chat.id, format!("Setting project directory to `{}` and creating a new session...", dir)).await?;
    
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
            // Revert state
            if let Some(sid) = prev_session_id {
                dialogue.update(State::ActiveSession { session_id: sid, directory: None, model }).await?;
            } else {
                dialogue.update(State::Start).await?;
            }
        }
    }
    
    Ok(())
}

async fn model_command(bot: Bot, msg: Message, dialogue: MyDialogue, client: Client, server_url: Arc<String>) -> HandlerResult {
    let state = dialogue.get().await?.unwrap_or_default();
    
    let (session_id, directory) = match state {
        State::ActiveSession { session_id, directory, .. } => (Some(session_id), directory),
        State::AwaitingProjectDir { prev_session_id, .. } => (prev_session_id, None),
        State::AwaitingModel { session_id, directory } => (session_id, directory),
        _ => (None, None),
    };

    // Fetch available models from OpenCode
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

    let keyboard = KeyboardMarkup::new(keyboard_rows)
        .resize_keyboard()
        .one_time_keyboard();

    bot.send_message(msg.chat.id, "Please select the model you want to use:")
        .reply_markup(keyboard)
        .await?;
        
    dialogue.update(State::AwaitingModel { session_id, directory }).await?;
    
    Ok(())
}

async fn receive_model(
    bot: Bot, 
    msg: Message, 
    dialogue: MyDialogue, 
    (session_id, directory): (Option<String>, Option<String>)
) -> HandlerResult {
    let model = msg.text().unwrap_or("").to_string();
    
    if let Some(sid) = session_id {
        bot.send_message(msg.chat.id, format!("✅ Active model set to: `{}`", model))
            .reply_markup(KeyboardRemove::new())
            .await?;
            
        dialogue.update(State::ActiveSession { 
            session_id: sid, 
            directory, 
            model: Some(model) 
        }).await?;
    } else {
        bot.send_message(msg.chat.id, "✅ Model set. Please use /start or /session to create a session first.")
            .reply_markup(KeyboardRemove::new())
            .await?;
            
        dialogue.update(State::Start).await?; 
    }
    
    Ok(())
}

async fn handle_no_session(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, "Please /start the bot or create a /session first.").await?;
    Ok(())
}

async fn handle_prompt(
    bot: Bot, 
    msg: Message, 
    (session_id, _directory, model): (String, Option<String>, Option<String>), 
    client: Client, 
    server_url: Arc<String>
) -> HandlerResult {
    let chat_id = msg.chat.id;
    let text = if let Some(t) = msg.text() { t } else { return Ok(()) };
    
    let bot_msg = match bot.send_message(chat_id, "⏳ Thinking...").await {
        Ok(m) => m,
        Err(e) => {
            error!("Failed to send thinking message: {}", e);
            return Ok(());
        }
    };
    
    // Set up SSE EventSource before sending the prompt
    let sse_req = client.get(format!("{}/event?sessionID={}", server_url, session_id)).try_clone().unwrap();
    let mut es = match EventSource::new(sse_req) {
        Ok(es) => es,
        Err(e) => {
            error!("Failed to create EventSource: {}", e);
            let _ = bot.edit_message_text(chat_id, bot_msg.id, format!("❌ Connection Error: {}", e)).await;
            return Ok(());
        }
    };

    // Construct prompt parts
    let mut payload = json!({
        "parts": [{"type": "text", "text": text}]
    });
    
    // Add model if specified
    if let Some(ref m) = model {
        if let Some((provider, model_id)) = m.split_once('/') {
            payload["model"] = json!({
                "providerID": provider,
                "modelID": model_id
            });
        }
    }

    // Send the prompt asynchronously
    let prompt_res = client.post(format!("{}/session/{}/prompt_async", server_url, session_id))
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
    
    let mut full_text = String::new();
    let mut last_edit = Instant::now();
    
    while let Some(event_res) = es.next().await {
        match event_res {
            Ok(Event::Open) => {
                info!("SSE Connection Opened!");
            }
            Ok(Event::Message(message)) => {
                if let Ok(data) = serde_json::from_str::<Value>(&message.data) {
                    let event_type = data["type"].as_str().unwrap_or("");
                    let properties = &data["properties"];
                    
                    match event_type {
                        "message.part.delta" => {
                            if let Some(delta) = properties["delta"].as_str() {
                                full_text.push_str(delta);
                                
                                // Edit message once per second to avoid rate limits
                                if last_edit.elapsed() > Duration::from_secs(1) {
                                    if !full_text.trim().is_empty() {
                                        let _ = bot.edit_message_text(chat_id, bot_msg.id, &full_text).await;
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
                            full_text.push_str(&format!("\n\n❌ Session Error: {}", err_msg));
                            break;
                        }
                        _ => {}
                    }
                }
            }
            Err(err) => {
                error!("SSE Stream Error: {}", err);
                full_text.push_str("\n\n⚠️ Stream interrupted.");
                break;
            }
        }
    }
    
    // Final flush and close SSE
    es.close();
    if !full_text.trim().is_empty() {
        let _ = bot.edit_message_text(chat_id, bot_msg.id, &full_text).await;
    } else {
        let _ = bot.edit_message_text(chat_id, bot_msg.id, "✅ Done (No text output)").await;
    }
    
    Ok(())
}

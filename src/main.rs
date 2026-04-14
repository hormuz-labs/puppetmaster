mod state;
mod helpers;
mod handlers;
mod markdown;

use std::env;
use std::sync::Arc;

use reqwest::Client;
use teloxide::{
    dispatching::{dialogue, dialogue::InMemStorage, UpdateHandler},
    prelude::*,
};
use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;

use crate::state::{State, Command};
use crate::handlers::*;

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
    use teloxide::utils::command::BotCommands;
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
        .branch(case![Command::Model].endpoint(model_command))
        .branch(case![Command::Abort].endpoint(abort_command))
        .branch(case![Command::Fetch(path)].endpoint(fetch_command))
        ;

    let message_handler = Update::filter_message()
        .branch(command_handler)
        // Interpret menu button clicks as commands
        .branch(case![State::AwaitingProjectDir { prev_session_id, prev_directory, model }].endpoint(receive_project_dir))
        .branch(case![State::AwaitingModel { session_id, directory }].endpoint(receive_model))
        .branch(dptree::filter(|msg: Message| msg.text() == Some("🔄 New Session")).endpoint(session_command_text))
        .branch(dptree::filter(|msg: Message| msg.text() == Some("📁 Set Project")).endpoint(project_command_text))
        .branch(dptree::filter(|msg: Message| msg.text() == Some("🤖 Change Model")).endpoint(model_command_text))
        .branch(dptree::filter(|msg: Message| msg.text() == Some("❓ Help")).endpoint(help_command))
        .branch(dptree::filter(|msg: Message| msg.text().unwrap_or("").starts_with("!")).endpoint(bash_command))
        .branch(case![State::ActiveSession { session_id, directory, model }].endpoint(handle_prompt))
        .branch(dptree::endpoint(handle_no_session));

    dialogue::enter::<Update, InMemStorage<State>, State, _>()
        .branch(message_handler)
}

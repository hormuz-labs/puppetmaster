mod state;
mod helpers;
mod handlers;
mod markdown;
mod config;
mod onboarding;

use std::env;
use std::sync::Arc;
use clap::{Parser, Subcommand};

use reqwest::Client;
use teloxide::{
    dispatching::{dialogue, dialogue::InMemStorage, UpdateHandler},
    prelude::*,
};
use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;

use crate::state::{State, Command};
use crate::handlers::*;
use crate::config::AppConfig;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Setup the bot configuration
    Onboard,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    if let Some(Commands::Onboard) = cli.command {
        onboarding::run_onboarding().await?;
        return Ok(());
    }

    // Initialize tracing
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default subscriber failed");

    // Load environment variables
    // 1. Try ~/.puppetmaster/.env
    if let Some(config_path) = onboarding::get_config_path() {
        if config_path.exists() {
            dotenvy::from_path(&config_path).ok();
        }
    }
    // 2. Fallback/override with local .env
    dotenvy::dotenv().ok();

    let bot_token = env::var("TELOXIDE_TOKEN").expect("TELOXIDE_TOKEN must be set. Run with 'onboard' command to setup.");
    let server_url = env::var("OPENCODE_SERVER_URL").unwrap_or_else(|_| "http://127.0.0.1:4096".to_string());
    
    // Load config
    let app_config = Arc::new(AppConfig::from_env());
    
    info!("Starting OpenCode Telegram Bot (Stage 3)...");
    
    let bot = Bot::new(bot_token);
    
    // Get bot info (needed for group mentions)
    let me = bot.get_me().await.expect("Failed to get bot info");
    let me = Arc::new(me);
    
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
            server_url,
            app_config,
            me
        ])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    Ok(())
}

async fn access_control_filter(
    bot: Bot,
    msg: Message,
    config: Arc<AppConfig>,
    me: Arc<teloxide::types::Me>,
) -> bool {
    // First check group rules (allows us to silently ignore non-targeted group chat messages)
    if msg.chat.is_group() || msg.chat.is_supergroup() {
        if !config.allow_in_groups {
            return false;
        }

        let is_mentioned = msg.text()
            .map(|t| t.contains(&format!("@{}", me.user.username.as_deref().unwrap_or(""))))
            .unwrap_or(false);
            
        let is_reply_to_bot = msg.reply_to_message()
            .and_then(|m| m.from.as_ref())
            .map(|u| u.id == me.user.id)
            .unwrap_or(false);

        // Allow commands starting with '/' that mention the bot (e.g. /start@bot)
        let is_command_for_bot = msg.text()
            .map(|t| t.starts_with('/') && t.contains(&format!("@{}", me.user.username.as_deref().unwrap_or(""))))
            .unwrap_or(false);

        if !is_mentioned && !is_reply_to_bot && !is_command_for_bot {
            return false;
        }
    }

    // Now check if the user is authorized to interact with the bot
    let user_id = match msg.from.as_ref() {
        Some(user) => user.id.0,
        None => return false,
    };

    let is_authorized = match &config.allowed_users {
        crate::config::AccessControl::All => true,
        crate::config::AccessControl::Restricted(ids) => ids.contains(&user_id),
    };

    if !is_authorized {
        let _ = bot.send_message(msg.chat.id, &config.unauthorized_message).await;
        return false;
    }

    true
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
        .branch(case![Command::ListSessions].endpoint(list_sessions_command))
        .branch(case![Command::Fetch(path)].endpoint(fetch_command))
        ;

    let message_handler = Update::filter_message()
        .filter_async(access_control_filter)
        .branch(command_handler)
        // Interpret menu button clicks as commands
        .branch(case![State::AwaitingProjectDir { prev_session_id, prev_directory, model }].endpoint(receive_project_dir))
        .branch(case![State::AwaitingModel { session_id, directory }].endpoint(receive_model))
        .branch(case![State::AwaitingSessionSelection { prev_session_id, prev_directory, prev_model }].endpoint(receive_session_selection))
        .branch(dptree::filter(|msg: Message| msg.text() == Some("🔄 New Session")).endpoint(session_command_text))
        .branch(dptree::filter(|msg: Message| msg.text() == Some("📁 Set Project")).endpoint(project_command_text))
        .branch(dptree::filter(|msg: Message| msg.text() == Some("🤖 Change Model")).endpoint(model_command_text))
        .branch(dptree::filter(|msg: Message| msg.text() == Some("📜 List Sessions")).endpoint(list_sessions_command_text))
        .branch(dptree::filter(|msg: Message| msg.text() == Some("❓ Help")).endpoint(help_command))
        .branch(dptree::filter(|msg: Message| msg.text().unwrap_or("").starts_with("!")).endpoint(bash_command))
        .branch(case![State::ActiveSession { session_id, directory, model }].endpoint(handle_prompt))
        .branch(dptree::endpoint(handle_no_session));

    dialogue::enter::<Update, InMemStorage<State>, State, _>()
        .branch(message_handler)
}

// Initialize rust_i18n first
rust_i18n::i18n!("locales", fallback = "en");

mod bot;
mod config;
mod error;
mod i18n;
mod opencode;
mod scheduled_task;
mod settings;

use crate::bot::{build_handler, init_bot, BotState};
use crate::config::AppConfig;
use crate::error::Result;
use crate::opencode::OpenCodeClient;
use crate::scheduled_task::TaskScheduler;
use crate::settings::SettingsManager;
use directories::ProjectDirs;
use std::path::PathBuf;
use std::sync::Arc;
use teloxide::dispatching::Dispatcher;
use tracing::{error, info, warn, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // Load configuration
    let config = match AppConfig::from_env() {
        Ok(c) => {
            if let Err(e) = c.validate() {
                eprintln!("Configuration error: {}", e);
                std::process::exit(1);
            }
            c
        }
        Err(e) => {
            eprintln!("Failed to load configuration: {}", e);
            std::process::exit(1);
        }
    };

    // Initialize logging
    let log_level = match config.server.log_level.as_str() {
        "debug" => Level::DEBUG,
        "warn" => Level::WARN,
        "error" => Level::ERROR,
        _ => Level::INFO,
    };

    let subscriber = FmtSubscriber::builder()
        .with_max_level(log_level)
        .with_target(false)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set tracing subscriber");

    info!("Starting OpenCode Telegram Bot v{}", env!("CARGO_PKG_VERSION"));
    info!("Allowed User ID: {}", config.telegram.allowed_user_id);

    // Set locale
    i18n::set_locale(&config.bot.locale);
    info!("Locale set to: {}", i18n::get_locale());

    // Get database path
    let db_path = get_database_path()?;
    info!("Database path: {:?}", db_path);

    // Initialize settings manager
    let settings = SettingsManager::new(&db_path).await?;
    info!("Settings manager initialized");

    // Initialize OpenCode client
    let opencode = OpenCodeClient::new(&config.opencode)?;
    info!("OpenCode client initialized");

    // Check OpenCode server health
    match opencode.health_check().await {
        Ok(true) => info!("OpenCode server is online"),
        Ok(false) => warn!("OpenCode server is offline - will retry on requests"),
        Err(e) => warn!("Could not check OpenCode server health: {}", e),
    }

    // Clone what we need for the scheduler
    let settings_for_scheduler = settings.clone();
    
    // Initialize task scheduler
    let scheduler = TaskScheduler::new(
        settings_for_scheduler.get_db_pool(),
        config.bot.clone(),
    ).await?;
    
    scheduler.initialize().await?;
    info!("Task scheduler initialized");

    // Initialize bot
    let bot = init_bot(
        config.clone(),
        settings.clone(),
        opencode.clone(),
        scheduler.clone(),
    ).await?;

    // Create bot state - all wrapped in Arc
    let state = BotState::new(config, settings, opencode, scheduler);
    
    // Clone the scheduler for shutdown
    let scheduler_for_shutdown = state.scheduler.clone();

    info!("Bot initialization complete. Starting dispatch...");

    // Build and run dispatcher
    let handler = build_handler();
    
    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![state])
        .build()
        .dispatch()
        .await;

    info!("Bot shutting down gracefully");
    
    // Cleanup
    if let Err(e) = scheduler_for_shutdown.shutdown().await {
        error!("Error shutting down scheduler: {}", e);
    }

    Ok(())
}

/// Get the database path based on platform
fn get_database_path() -> Result<PathBuf> {
    let proj_dirs = ProjectDirs::from("com", "opencode", "telegram-bot")
        .ok_or_else(|| error::BotError::Config(
            "Could not determine project directories".to_string()
        ))?;

    let data_dir = proj_dirs.data_dir();
    std::fs::create_dir_all(data_dir)?;

    Ok(data_dir.join("bot.db"))
}

// Re-export for tests
pub use error::{BotError, Result as BotResult};

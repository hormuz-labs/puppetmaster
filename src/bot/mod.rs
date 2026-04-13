use crate::config::AppConfig;
use crate::error::Result;
use crate::i18n::{t, I18nKey};
use crate::opencode::OpenCodeClient;
use crate::scheduled_task::TaskScheduler;
use crate::settings::SettingsManager;
use std::sync::Arc;
use teloxide::dispatching::{HandlerExt, UpdateFilterExt};
use teloxide::dptree;
use teloxide::prelude::*;
use teloxide::types::{BotCommand, ParseMode};
use teloxide::utils::command::BotCommands;
use tracing::{debug, error, info};

pub mod commands;
pub mod handlers;
pub mod middleware;

use commands::*;
use handlers::*;
use middleware::*;

/// Bot state shared across handlers
#[derive(Clone)]
pub struct BotState {
    pub config: Arc<AppConfig>,
    pub settings: Arc<SettingsManager>,
    pub opencode: Arc<OpenCodeClient>,
    pub scheduler: Arc<TaskScheduler>,
}

impl BotState {
    pub fn new(
        config: AppConfig,
        settings: SettingsManager,
        opencode: OpenCodeClient,
        scheduler: TaskScheduler,
    ) -> Self {
        Self {
            config: Arc::new(config),
            settings: Arc::new(settings),
            opencode: Arc::new(opencode),
            scheduler: Arc::new(scheduler),
        }
    }
}

/// Bot commands
#[derive(BotCommands, Clone, Debug)]
#[command(
    rename_rule = "lowercase",
    description = "OpenCode Telegram Bot commands:"
)]
pub enum Command {
    #[command(description = "Start the bot")]
    Start,
    #[command(description = "Show help and available commands")]
    Help,
    #[command(description = "Show server and session status")]
    Status,
    #[command(description = "Create a new session")]
    New,
    #[command(description = "List and switch sessions")]
    Sessions,
    #[command(description = "List and switch projects")]
    Projects,
    #[command(description = "Run custom commands")]
    Commands,
    #[command(description = "Abort current task")]
    Abort,
    #[command(description = "Create scheduled task")]
    Task,
    #[command(description = "List scheduled tasks")]
    Tasklist,
    #[command(description = "Toggle text-to-speech")]
    Tts,
    #[command(description = "Rename current session")]
    Rename,
    #[command(description = "Start OpenCode server")]
    OpencodeStart,
    #[command(description = "Stop OpenCode server")]
    OpencodeStop,
}

/// Initialize the Telegram bot
pub async fn init_bot(
    config: AppConfig,
    settings: SettingsManager,
    opencode: OpenCodeClient,
    scheduler: TaskScheduler,
) -> Result<Bot> {
    let bot = Bot::new(&config.telegram.token);
    
    // Set up command descriptions
    let commands = vec![
        BotCommand::new("start", t(I18nKey::CommandsStartDesc, None)),
        BotCommand::new("help", t(I18nKey::CommandsHelpDesc, None)),
        BotCommand::new("status", t(I18nKey::CommandsStatusDesc, None)),
        BotCommand::new("new", t(I18nKey::CommandsNewDesc, None)),
        BotCommand::new("sessions", t(I18nKey::CommandsSessionsDesc, None)),
        BotCommand::new("projects", t(I18nKey::CommandsProjectsDesc, None)),
        BotCommand::new("commands", t(I18nKey::CommandsCommandsDesc, None)),
        BotCommand::new("abort", t(I18nKey::CommandsAbortDesc, None)),
        BotCommand::new("task", t(I18nKey::CommandsTaskDesc, None)),
        BotCommand::new("tasklist", t(I18nKey::CommandsTasklistDesc, None)),
        BotCommand::new("tts", t(I18nKey::CommandsTtsDesc, None)),
        BotCommand::new("rename", t(I18nKey::CommandsRenameDesc, None)),
        BotCommand::new("opencode_start", t(I18nKey::CommandsOpencodeStartDesc, None)),
        BotCommand::new("opencode_stop", t(I18nKey::CommandsOpencodeStopDesc, None)),
    ];

    // Set commands for the bot (global)
    if let Err(e) = bot.set_my_commands(commands).await {
        tracing::warn!("Failed to set bot commands: {}", e);
    }

    info!("Telegram bot initialized");
    Ok(bot)
}

/// Build the update handler
pub fn build_handler() -> teloxide::dispatching::UpdateHandler<crate::error::BotError> {
    dptree::entry()
        // Log all updates
        .inspect(|u: Update| {
            debug!("Received update: {:?}", u.id);
        })
        // Filter messages
        .branch(Update::filter_message().chain(
            dptree::entry()
                // Auth middleware - check user ID
                .filter_async(auth_middleware)
                // Handle commands
                .branch(
                    dptree::entry()
                        .filter_command::<Command>()
                        .endpoint(handle_command),
                )
                // Handle text messages (prompts)
                .branch(
                    dptree::filter(|msg: Message| msg.text().is_some())
                        .endpoint(handle_text_message),
                )
                // Handle voice messages
                .branch(
                    dptree::filter(|msg: Message| msg.voice().is_some())
                        .endpoint(handle_voice_message),
                )
                // Handle documents
                .branch(
                    dptree::filter(|msg: Message| msg.document().is_some())
                        .endpoint(handle_document_message),
                ),
        ))
        // Handle callback queries
        .branch(Update::filter_callback_query().chain(
            dptree::entry()
                .filter_async(auth_callback_middleware)
                .endpoint(handle_callback_query),
        ))
}

/// Handle bot commands
async fn handle_command(
    bot: Bot,
    msg: Message,
    cmd: Command,
    state: BotState,
) -> Result<()> {
    info!("Received command {:?} from user {:?}", cmd, msg.from().map(|u| u.id));

    match cmd {
        Command::Start => cmd_start(bot, msg).await,
        Command::Help => cmd_help(bot, msg).await,
        Command::Status => cmd_status(bot, msg, state).await,
        Command::New => cmd_new(bot, msg, state).await,
        Command::Sessions => cmd_sessions(bot, msg, state).await,
        Command::Projects => cmd_projects(bot, msg, state).await,
        Command::Commands => cmd_commands(bot, msg).await,
        Command::Abort => cmd_abort(bot, msg, state).await,
        Command::Task => cmd_task(bot, msg, state).await,
        Command::Tasklist => cmd_tasklist(bot, msg, state).await,
        Command::Tts => cmd_tts(bot, msg, state).await,
        Command::Rename => cmd_rename(bot, msg, state).await,
        Command::OpencodeStart => cmd_opencode_start(bot, msg).await,
        Command::OpencodeStop => cmd_opencode_stop(bot, msg).await,
    }
}

/// Helper function to send a reply
pub async fn reply(bot: &Bot, chat_id: ChatId, text: impl Into<String>) -> Result<()> {
    bot.send_message(chat_id, text)
        .parse_mode(ParseMode::MarkdownV2)
        .await?;
    Ok(())
}

/// Escape markdown characters for Telegram MarkdownV2
pub fn escape_markdown(text: &str) -> String {
    text.replace('\\', "\\\\")
        .replace('_', "\\_")
        .replace('*', "\\*")
        .replace('[', "\\[")
        .replace(']', "\\]")
        .replace('(', "\\(")
        .replace(')', "\\)")
        .replace('~', "\\~")
        .replace('`', "\\`")
        .replace('>', "\\>")
        .replace('#', "\\#")
        .replace('+', "\\+")
        .replace('-', "\\-")
        .replace('=', "\\=")
        .replace('|', "\\|")
        .replace('{', "\\{")
        .replace('}', "\\}")
        .replace('.', "\\.")
        .replace('!', "\\!")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_markdown() {
        let input = "Hello _world_ *bold* [link](url)";
        let escaped = escape_markdown(input);
        assert!(escaped.contains("\\_"));
        assert!(escaped.contains("\\*"));
        assert!(escaped.contains("\\["));
    }
}

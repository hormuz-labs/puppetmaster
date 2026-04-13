use super::{escape_markdown, BotState};
use crate::i18n::{t, I18nKey};
use crate::error::Result;
use teloxide::prelude::*;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, ParseMode};
use tracing::{error, info};

/// /start command
pub async fn cmd_start(bot: Bot, msg: Message) -> Result<()> {
    let welcome = t(I18nKey::BotWelcome, None);
    bot.send_message(msg.chat.id, welcome)
        .parse_mode(ParseMode::MarkdownV2)
        .await?;
    Ok(())
}

/// /help command
pub async fn cmd_help(bot: Bot, msg: Message) -> Result<()> {
    let header = escape_markdown(&t(I18nKey::CommandsHelpHeader, None));
    let start_desc = escape_markdown(&t(I18nKey::CommandsStartDesc, None));
    let help_desc = escape_markdown(&t(I18nKey::CommandsHelpDesc, None));
    let status_desc = escape_markdown(&t(I18nKey::CommandsStatusDesc, None));
    let new_desc = escape_markdown(&t(I18nKey::CommandsNewDesc, None));
    let sessions_desc = escape_markdown(&t(I18nKey::CommandsSessionsDesc, None));
    let projects_desc = escape_markdown(&t(I18nKey::CommandsProjectsDesc, None));
    let commands_desc = escape_markdown(&t(I18nKey::CommandsCommandsDesc, None));
    let abort_desc = escape_markdown(&t(I18nKey::CommandsAbortDesc, None));
    let task_desc = escape_markdown(&t(I18nKey::CommandsTaskDesc, None));
    let tasklist_desc = escape_markdown(&t(I18nKey::CommandsTasklistDesc, None));
    let tts_desc = escape_markdown(&t(I18nKey::CommandsTtsDesc, None));
    let rename_desc = escape_markdown(&t(I18nKey::CommandsRenameDesc, None));
    let start_server_desc = escape_markdown(&t(I18nKey::CommandsOpencodeStartDesc, None));
    let stop_server_desc = escape_markdown(&t(I18nKey::CommandsOpencodeStopDesc, None));
    
    let help_text = format!(
        "📖 *{}*\n\n/start \\= {}\n/help \\= {}\n/status \\= {}\n/new \\= {}\n/sessions \\= {}\n/projects \\= {}\n/commands \\= {}\n/abort \\= {}\n/task \\= {}\n/tasklist \\= {}\n/tts \\= {}\n/rename \\= {}\n/opencode_start \\= {}\n/opencode_stop \\= {}",
        header, start_desc, help_desc, status_desc, new_desc, sessions_desc,
        projects_desc, commands_desc, abort_desc, task_desc, tasklist_desc,
        tts_desc, rename_desc, start_server_desc, stop_server_desc
    );

    bot.send_message(msg.chat.id, help_text)
        .parse_mode(ParseMode::MarkdownV2)
        .await?;
    Ok(())
}

/// /status command
pub async fn cmd_status(bot: Bot, msg: Message, state: BotState) -> Result<()> {
    // Check server health
    let server_online = state.opencode.health_check().await.unwrap_or(false);
    
    let current_session = state.settings.current_session();
    let current_project = state.settings.current_project();
    let current_model = state.settings.current_model();

    let mut status_parts = vec![
        format!("📊 *{}*", escape_markdown(&t(I18nKey::StatusHeader, None))),
        String::new(),
    ];

    // Server status
    if server_online {
        status_parts.push(format!("🟢 {}", escape_markdown(&t(I18nKey::StatusServerOnline, None))));
    } else {
        status_parts.push(format!("🔴 {}", escape_markdown(&t(I18nKey::StatusServerOffline, None))));
    }

    status_parts.push(String::new());

    // Current project
    match current_project {
        Some(project) => {
            let name = project.name.as_ref().unwrap_or(&project.worktree);
            status_parts.push(format!(
                "📁 {}: {}",
                escape_markdown(&t(I18nKey::StatusCurrentProject, None)),
                escape_markdown(name)
            ));
        }
        None => {
            status_parts.push(format!(
                "📁 {}: _{}_",
                escape_markdown(&t(I18nKey::StatusCurrentProject, None)),
                escape_markdown(&t(I18nKey::StatusNoProject, None))
            ));
        }
    }

    // Current session
    match current_session {
        Some(session) => {
            status_parts.push(format!(
                "💬 {}: {}",
                escape_markdown(&t(I18nKey::StatusCurrentSession, None)),
                escape_markdown(&session.title)
            ));
        }
        None => {
            status_parts.push(format!(
                "💬 {}: _{}_",
                escape_markdown(&t(I18nKey::StatusCurrentSession, None)),
                escape_markdown(&t(I18nKey::StatusNoSession, None))
            ));
        }
    }

    // Current model
    match current_model {
        Some(model) => {
            status_parts.push(format!(
                "🤖 {}: `{}`",
                escape_markdown(&t(I18nKey::StatusCurrentModel, None)),
                escape_markdown(&model.to_string())
            ));
        }
        None => {
            status_parts.push(format!(
                "🤖 {}: `{}`",
                escape_markdown(&t(I18nKey::StatusCurrentModel, None)),
                escape_markdown(&format!("{}/{}", 
                    state.config.opencode.model_provider,
                    state.config.opencode.model_id
                ))
            ));
        }
    }

    let status_text = status_parts.join("\n");

    bot.send_message(msg.chat.id, status_text)
        .parse_mode(ParseMode::MarkdownV2)
        .await?;

    Ok(())
}

/// /new command - create new session
pub async fn cmd_new(bot: Bot, msg: Message, state: BotState) -> Result<()> {
    // Show processing message
    let processing_msg = bot
        .send_message(msg.chat.id, t(I18nKey::BotProcessing, None))
        .await?;

    // Create new session via OpenCode API
    match state.opencode.create_session(None).await {
        Ok(session) => {
            // Save as current session
            state.settings.set_current_session(session.clone()).await?;

            // Delete processing message
            bot.delete_message(msg.chat.id, processing_msg.id).await.ok();

            // Send success message
            let success_text = t(I18nKey::BotSessionCreated, None);
            bot.send_message(msg.chat.id, success_text).await?;

            info!("Created new session: {}", session.id);
        }
        Err(e) => {
            error!("Failed to create session: {}", e);
            
            bot.delete_message(msg.chat.id, processing_msg.id).await.ok();
            
            let error_text = format!("❌ Failed to create session: {}", e);
            bot.send_message(msg.chat.id, error_text).await?;
        }
    }

    Ok(())
}

/// /sessions command - list and switch sessions
pub async fn cmd_sessions(bot: Bot, msg: Message, state: BotState) -> Result<()> {
    match state.opencode.list_sessions().await {
        Ok(sessions) => {
            if sessions.is_empty() {
                bot.send_message(msg.chat.id, t(I18nKey::SessionNoSessions, None))
                    .await?;
                return Ok(());
            }

            let header = format!("💬 *{}*", escape_markdown(&t(I18nKey::SessionListHeader, None)));
            
            // Build inline keyboard with sessions
            let mut buttons: Vec<Vec<InlineKeyboardButton>> = sessions
                .into_iter()
                .take(state.config.bot.sessions_list_limit)
                .map(|s| {
                    vec![InlineKeyboardButton::callback(
                        format!("{}", s.title),
                        format!("session:{}", s.id),
                    )]
                })
                .collect();

            // Add back button
            buttons.push(vec![InlineKeyboardButton::callback(
                t(I18nKey::CommonBack, None),
                "menu:main",
            )]);

            let keyboard = InlineKeyboardMarkup::new(buttons);

            bot.send_message(msg.chat.id, format!("{}\n\n{}", header, 
                escape_markdown(&t(I18nKey::SessionSelectPrompt, None))))
                .parse_mode(ParseMode::MarkdownV2)
                .reply_markup(keyboard)
                .await?;
        }
        Err(e) => {
            error!("Failed to list sessions: {}", e);
            bot.send_message(msg.chat.id, format!("❌ Error: {}", e))
                .await?;
        }
    }

    Ok(())
}

/// /projects command - list and switch projects
pub async fn cmd_projects(bot: Bot, msg: Message, state: BotState) -> Result<()> {
    match state.opencode.list_projects().await {
        Ok(projects) => {
            if projects.is_empty() {
                bot.send_message(msg.chat.id, t(I18nKey::ProjectNoProjects, None))
                    .await?;
                return Ok(());
            }

            let header = format!("📁 *{}*", escape_markdown(&t(I18nKey::ProjectListHeader, None)));
            
            // Build inline keyboard with projects
            let mut buttons: Vec<Vec<InlineKeyboardButton>> = projects
                .into_iter()
                .take(state.config.bot.projects_list_limit)
                .map(|p| {
                    let name = p.name.as_ref().unwrap_or(&p.worktree);
                    vec![InlineKeyboardButton::callback(
                        name.to_string(),
                        format!("project:{}", p.id),
                    )]
                })
                .collect();

            buttons.push(vec![InlineKeyboardButton::callback(
                t(I18nKey::CommonBack, None),
                "menu:main",
            )]);

            let keyboard = InlineKeyboardMarkup::new(buttons);

            bot.send_message(msg.chat.id, format!("{}\n\n{}", header,
                escape_markdown(&t(I18nKey::ProjectSelectPrompt, None))))
                .parse_mode(ParseMode::MarkdownV2)
                .reply_markup(keyboard)
                .await?;
        }
        Err(e) => {
            error!("Failed to list projects: {}", e);
            bot.send_message(msg.chat.id, format!("❌ Error: {}", e))
                .await?;
        }
    }

    Ok(())
}

/// /commands command - show custom commands (placeholder)
pub async fn cmd_commands(bot: Bot, msg: Message) -> Result<()> {
    bot.send_message(msg.chat.id, "🛠 Custom commands will be implemented soon!")
        .await?;
    Ok(())
}

/// /abort command - abort current session operation
pub async fn cmd_abort(bot: Bot, msg: Message, state: BotState) -> Result<()> {
    match state.settings.current_session() {
        Some(session) => {
            match state.opencode.abort_session(&session.id).await {
                Ok(_) => {
                    bot.send_message(msg.chat.id, "✅ Current operation aborted").await?;
                }
                Err(e) => {
                    bot.send_message(msg.chat.id, format!("❌ Failed to abort: {}", e))
                        .await?;
                }
            }
        }
        None => {
            bot.send_message(msg.chat.id, t(I18nKey::BotNoActiveSession, None))
                .await?;
        }
    }
    Ok(())
}

/// /task command - create scheduled task (simplified)
pub async fn cmd_task(bot: Bot, msg: Message, _state: BotState) -> Result<()> {
    // For now, just show task creation help
    let help_text = "⏰ Task Creation\n\nUsage: Send me a message in this format:\n/task <schedule> <prompt>\n\nExamples:\n• One-time: `/task 2024-12-25 10:00 Fix the login bug`\n• Cron: `/task 0 9 * * * Daily code review`\n\nOr use /tasklist to manage existing tasks.";

    bot.send_message(msg.chat.id, help_text)
        .parse_mode(ParseMode::MarkdownV2)
        .await?;
    Ok(())
}

/// /tasklist command - list scheduled tasks
pub async fn cmd_tasklist(bot: Bot, msg: Message, state: BotState) -> Result<()> {
    let tasks = state.scheduler.list_tasks().await;

    if tasks.is_empty() {
        bot.send_message(msg.chat.id, t(I18nKey::TaskNoTasks, None))
            .await?;
        return Ok(());
    }

    let mut text_parts = vec![format!("⏰ *{}*\n", escape_markdown(&t(I18nKey::TaskHeader, None)))];

    for task in tasks {
        let status_emoji = match task.last_status {
            crate::scheduled_task::TaskStatus::Idle => "⏸️",
            crate::scheduled_task::TaskStatus::Running => "🔄",
            crate::scheduled_task::TaskStatus::Success => "✅",
            crate::scheduled_task::TaskStatus::Error => "❌",
        };

        let kind_emoji = match task.kind {
            crate::scheduled_task::TaskKind::Once => "📅",
            crate::scheduled_task::TaskKind::Cron => "🔄",
        };

        let summary = format!(
            "{} {} *{}*\n   📝 {}\n   📅 {}\n   📊 Runs: {}\n",
            status_emoji,
            kind_emoji,
            escape_markdown(&task.prompt.chars().take(30).collect::<String>()),
            escape_markdown(&task.schedule_summary),
            task.next_run_at.map(|d| d.to_string()).unwrap_or_else(|| "Not scheduled".to_string()),
            task.run_count
        );
        text_parts.push(summary);
    }

    let text = text_parts.join("\n");
    bot.send_message(msg.chat.id, text)
        .parse_mode(ParseMode::MarkdownV2)
        .await?;

    Ok(())
}

/// /tts command - toggle text-to-speech
pub async fn cmd_tts(bot: Bot, msg: Message, state: BotState) -> Result<()> {
    let current = state.settings.tts_enabled();
    let new_state = !current;

    match state.settings.set_tts_enabled(new_state).await {
        Ok(_) => {
            let message = if new_state {
                t(I18nKey::TtsEnabled, None)
            } else {
                t(I18nKey::TtsDisabled, None)
            };
            bot.send_message(msg.chat.id, message).await?;
        }
        Err(e) => {
            bot.send_message(msg.chat.id, format!("❌ Error: {}", e))
                .await?;
        }
    }

    Ok(())
}

/// /rename command - rename current session
pub async fn cmd_rename(bot: Bot, msg: Message, _state: BotState) -> Result<()> {
    // For now, prompt user to send new name
    bot.send_message(
        msg.chat.id,
        "✏️ Send me the new name for the current session:",
    )
    .await?;

    // In a full implementation, you'd set up a dialogue state here
    // to capture the next message as the new name

    Ok(())
}

/// /opencode_start command - start OpenCode server (placeholder)
pub async fn cmd_opencode_start(bot: Bot, msg: Message) -> Result<()> {
    bot.send_message(
        msg.chat.id,
        "🚀 Starting OpenCode server...\n\n(Note: This is a placeholder. Start the server manually with `opencode serve`)",
    )
    .parse_mode(ParseMode::MarkdownV2)
    .await?;
    Ok(())
}

/// /opencode_stop command - stop OpenCode server (placeholder)
pub async fn cmd_opencode_stop(bot: Bot, msg: Message) -> Result<()> {
    bot.send_message(
        msg.chat.id,
        "🛑 Stopping OpenCode server...\n\n(Note: This is a placeholder.)",
    )
    .parse_mode(ParseMode::MarkdownV2)
    .await?;
    Ok(())
}

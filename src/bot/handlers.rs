use super::BotState;
use crate::i18n::{t, I18nKey};
use crate::error::Result;
use teloxide::prelude::*;

use tracing::{debug, error, info};

/// Handle text messages (prompts)
pub async fn handle_text_message(
    bot: Bot,
    msg: Message,
    state: BotState,
) -> Result<()> {
    let text = msg.text().unwrap_or_default();
    
    debug!("Received text message: {}", text);

    // Check if there's an active session
    let session = match state.settings.current_session() {
        Some(s) => s,
        None => {
            bot.send_message(msg.chat.id, t(I18nKey::BotNoActiveSession, None))
                .await?;
            return Ok(());
        }
    };

    // Show thinking indicator
    let thinking_msg = bot
        .send_message(msg.chat.id, t(I18nKey::BotThinking, None))
        .await?;

    // Send prompt to OpenCode
    match state.opencode.send_prompt(&session.id, text).await {
        Ok(_) => {
            // The response will come via SSE events
            // For now, just delete the thinking message
            bot.delete_message(msg.chat.id, thinking_msg.id).await.ok();
            
            info!("Sent prompt to session {}: {}", session.id, 
                text.chars().take(50).collect::<String>());
        }
        Err(e) => {
            bot.delete_message(msg.chat.id, thinking_msg.id).await.ok();
            
            let mut params = std::collections::HashMap::new();
            let error_msg = e.to_string();
            params.insert("message", error_msg);
            let error_text = t(I18nKey::BotSessionError, Some(params));
            bot.send_message(msg.chat.id, error_text).await?;
        }
    }

    Ok(())
}

/// Handle voice messages
pub async fn handle_voice_message(
    bot: Bot,
    msg: Message,
    state: BotState,
) -> Result<()> {
    // Check if STT is configured
    if state.config.stt.is_none() {
        bot.send_message(msg.chat.id, t(I18nKey::SttNotConfigured, None))
            .await?;
        return Ok(());
    }

    let voice = msg.voice().unwrap();
    
    bot.send_message(msg.chat.id, t(I18nKey::SttTranscribing, None))
        .await?;

    // Get file info
    let file_id = voice.file.id.clone();
    
    // Download voice file via Telegram API
    let _file = match bot.get_file(file_id).await {
        Ok(f) => f,
        Err(e) => {
            error!("Failed to get voice file: {}", e);
            bot.send_message(msg.chat.id, t(I18nKey::SttTranscriptionFailed, None))
                .await?;
            return Ok(());
        }
    };
    
    // TODO: Download file content and send to STT API
    // For now, just acknowledge receipt
    bot.send_message(
        msg.chat.id,
        "🎤 Voice message received. STT processing would happen here.",
    )
    .await?;

    Ok(())
}

/// Handle document/file messages
pub async fn handle_document_message(
    bot: Bot,
    msg: Message,
    state: BotState,
) -> Result<()> {
    let document = msg.document().unwrap();
    
    // Check file size
    let file_size = document.file.size;
    if file_size > state.config.files.max_file_size_kb as u32 * 1024 {
        bot.send_message(
            msg.chat.id,
            format!(
                "❌ File too large. Max size: {} KB",
                state.config.files.max_file_size_kb
            ),
        )
        .await?;
        return Ok(());
    }

    // Check if there's an active session
    let session = match state.settings.current_session() {
        Some(s) => s,
        None => {
            bot.send_message(msg.chat.id, t(I18nKey::BotNoActiveSession, None))
                .await?;
            return Ok(());
        }
    };

    bot.send_message(msg.chat.id, format!("📄 Processing file: {}", document.file_name.as_deref().unwrap_or("unnamed")))
        .await?;

    // Get file info
    let file_id = document.file.id.clone();
    
    let _file = match bot.get_file(file_id).await {
        Ok(f) => f,
        Err(e) => {
            error!("Failed to get file: {}", e);
            bot.send_message(msg.chat.id, "❌ Failed to download file").await?;
            return Ok(());
        }
    };

    // TODO: Download file content and process
    // This would involve reading the file content and including it in the prompt

    info!(
        "Received file from user: {} for session {}",
        document.file_name.as_deref().unwrap_or("unnamed"),
        session.id
    );

    Ok(())
}

/// Handle callback queries (inline button presses)
pub async fn handle_callback_query(
    bot: Bot,
    q: CallbackQuery,
    state: BotState,
) -> Result<()> {
    let data = q.data.as_deref().unwrap_or_default();
    
    // Extract chat_id and message_id from the message
    let (chat_id, message_id) = if let Some(msg) = &q.message {
        match msg {
            teloxide::types::MaybeInaccessibleMessage::Regular(m) => {
                (Some(m.chat.id), Some(m.id))
            }
            teloxide::types::MaybeInaccessibleMessage::Inaccessible(_) => {
                (None, None)
            }
        }
    } else {
        (None, None)
    };

    debug!("Received callback query: {}", data);

    // Answer the callback query to stop the loading animation
    bot.answer_callback_query(q.id.clone()).await?;

    let Some(chat_id) = chat_id else {
        return Ok(());
    };

    match data.split(':').collect::<Vec<_>>().as_slice() {
        ["session", session_id] => {
            handle_session_selection(bot, chat_id, message_id, state, session_id).await?;
        }
        ["project", project_id] => {
            handle_project_selection(bot, chat_id, message_id, state, project_id).await?;
        }
        ["menu", "main"] => {
            // Return to main menu
            if let Some(msg_id) = message_id {
                bot.delete_message(chat_id, msg_id).await.ok();
            }
            bot.send_message(chat_id, "👋 Back to main menu. Use /help for commands.")
                .await?;
        }
        _ => {
            debug!("Unknown callback data: {}", data);
        }
    }

    Ok(())
}

/// Handle session selection from inline keyboard
async fn handle_session_selection(
    bot: Bot,
    chat_id: ChatId,
    message_id: Option<teloxide::types::MessageId>,
    state: BotState,
    session_id: &str,
) -> Result<()> {
    // Get session info from OpenCode
    match state.opencode.get_session(session_id).await {
        Ok(session) => {
            // Save as current session
            state.settings.set_current_session(session.clone()).await?;

            // Delete the selection message
            if let Some(msg_id) = message_id {
                bot.delete_message(chat_id, msg_id).await.ok();
            }

            // Send confirmation
            let mut params = std::collections::HashMap::new();
            let title = session.title.clone();
            params.insert("title", title);
            let text = t(I18nKey::BotSessionSwitched, Some(params));
            bot.send_message(chat_id, text).await?;

            info!("Switched to session: {}", session_id);
        }
        Err(e) => {
            error!("Failed to get session {}: {}", session_id, e);
            bot.send_message(chat_id, format!("❌ Failed to switch session: {}", e))
                .await?;
        }
    }

    Ok(())
}

/// Handle project selection from inline keyboard
async fn handle_project_selection(
    bot: Bot,
    chat_id: ChatId,
    message_id: Option<teloxide::types::MessageId>,
    state: BotState,
    project_id: &str,
) -> Result<()> {
    // In a real implementation, you'd get project info from the list
    // For now, create a minimal project info
    let project = crate::opencode::types::ProjectInfo {
        id: project_id.to_string(),
        worktree: project_id.to_string(),
        name: None,
    };

    // Save as current project
    state.settings.set_current_project(project.clone()).await?;

    // Delete the selection message
    if let Some(msg_id) = message_id {
        bot.delete_message(chat_id, msg_id).await.ok();
    }

    // Send confirmation
    let project_name = project.name.as_ref().unwrap_or(&project.worktree).clone();
    let mut params = std::collections::HashMap::new();
    params.insert("name", project_name);
    let text = t(I18nKey::BotProjectSwitched, Some(params));
    bot.send_message(chat_id, text).await?;

    info!("Switched to project: {}", project_id);

    Ok(())
}

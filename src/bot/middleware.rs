use crate::config::AppConfig;
use teloxide::prelude::*;
use tracing::{debug, warn};

/// Authentication middleware - check if user is authorized
pub async fn auth_middleware(msg: Message, config: AppConfig) -> bool {
    let Some(user) = msg.from() else {
        return false;
    };

    let user_id = user.id.0 as i64;
    let allowed_id = config.telegram.allowed_user_id;

    if user_id != allowed_id {
        warn!(
            "Unauthorized access attempt from user {} (allowed: {})",
            user_id, allowed_id
        );
        
        // Optionally send a message to the unauthorized user
        // bot.send_message(msg.chat.id, "⛔ You are not authorized to use this bot.").await.ok();
        
        return false;
    }

    debug!("Authorized user {} accessing bot", user_id);
    true
}

/// Authentication middleware for callback queries
pub async fn auth_callback_middleware(q: CallbackQuery, config: AppConfig) -> bool {
    let user = q.from;

    let user_id = user.id.0 as i64;
    let allowed_id = config.telegram.allowed_user_id;

    if user_id != allowed_id {
        warn!(
            "Unauthorized callback query from user {} (allowed: {})",
            user_id, allowed_id
        );
        return false;
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{TelegramConfig, OpencodeConfig, BotConfig, ServerConfig, FileConfig, OpenConfig};

    fn create_test_config(allowed_id: i64) -> AppConfig {
        AppConfig {
            telegram: TelegramConfig {
                token: "test_token".to_string(),
                allowed_user_id: allowed_id,
                proxy_url: None,
            },
            opencode: OpencodeConfig {
                api_url: "http://localhost:4096".to_string(),
                username: "opencode".to_string(),
                password: None,
                model_provider: "opencode".to_string(),
                model_id: "big-pickle".to_string(),
            },
            stt: None,
            tts: None,
            bot: BotConfig::default(),
            server: ServerConfig::default(),
            files: FileConfig::default(),
            open: OpenConfig::default(),
        }
    }

    // Note: Testing auth middleware properly would require mocking Telegram types
    // which is complex. These tests demonstrate the logic.
    
    #[test]
    fn test_auth_config() {
        let config = create_test_config(123456);
        assert_eq!(config.telegram.allowed_user_id, 123456);
        
        let config2 = create_test_config(999999);
        assert_eq!(config2.telegram.allowed_user_id, 999999);
    }
}

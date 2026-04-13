use serde::Deserialize;
use std::str::FromStr;

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub telegram: TelegramConfig,
    pub opencode: OpencodeConfig,
    pub stt: Option<SttConfig>,
    pub tts: Option<TtsConfig>,
    #[serde(default)]
    pub bot: BotConfig,
    #[serde(default)]
    pub server: ServerConfig,
    #[serde(default)]
    pub files: FileConfig,
    #[serde(default)]
    pub open: OpenConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TelegramConfig {
    pub token: String,
    pub allowed_user_id: i64,
    pub proxy_url: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OpencodeConfig {
    #[serde(default = "default_opencode_url")]
    pub api_url: String,
    #[serde(default = "default_opencode_username")]
    pub username: String,
    pub password: Option<String>,
    pub model_provider: String,
    pub model_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SttConfig {
    pub api_url: String,
    pub api_key: String,
    #[serde(default = "default_stt_model")]
    pub model: String,
    pub language: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TtsConfig {
    pub api_url: String,
    pub api_key: String,
    #[serde(default = "default_tts_model")]
    pub model: String,
    #[serde(default = "default_tts_voice")]
    pub voice: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct BotConfig {
    pub sessions_list_limit: usize,
    pub projects_list_limit: usize,
    pub commands_list_limit: usize,
    pub task_limit: usize,
    pub response_stream_throttle_ms: u64,
    pub bash_tool_display_max_length: usize,
    pub locale: String,
    pub hide_thinking_messages: bool,
    pub hide_tool_call_messages: bool,
    pub message_format_mode: MessageFormatMode,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageFormatMode {
    Raw,
    Markdown,
}

impl Default for BotConfig {
    fn default() -> Self {
        Self {
            sessions_list_limit: 10,
            projects_list_limit: 10,
            commands_list_limit: 10,
            task_limit: 10,
            response_stream_throttle_ms: 500,
            bash_tool_display_max_length: 128,
            locale: "en".to_string(),
            hide_thinking_messages: false,
            hide_tool_call_messages: false,
            message_format_mode: MessageFormatMode::Markdown,
        }
    }
}

impl FromStr for MessageFormatMode {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "raw" => Ok(MessageFormatMode::Raw),
            "markdown" => Ok(MessageFormatMode::Markdown),
            _ => Err(format!("Invalid message format mode: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct ServerConfig {
    pub log_level: String,
    pub log_retention: usize,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            log_level: "info".to_string(),
            log_retention: 10,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct FileConfig {
    pub max_file_size_kb: usize,
}

impl Default for FileConfig {
    fn default() -> Self {
        Self {
            max_file_size_kb: 100,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct OpenConfig {
    pub browser_roots: Option<String>,
}

fn default_opencode_url() -> String {
    "http://localhost:4096".to_string()
}

fn default_opencode_username() -> String {
    "opencode".to_string()
}

fn default_stt_model() -> String {
    "whisper-large-v3-turbo".to_string()
}

fn default_tts_model() -> String {
    "gpt-4o-mini-tts".to_string()
}

fn default_tts_voice() -> String {
    "alloy".to_string()
}

impl AppConfig {
    pub fn from_env() -> crate::error::Result<Self> {
        dotenvy::dotenv().ok();

        let config = config::Config::builder()
            .add_source(config::Environment::default().separator("__"))
            .build()?;

        Ok(config.try_deserialize()?)
    }

    pub fn validate(&self) -> crate::error::Result<()> {
        if self.telegram.token.is_empty() {
            return Err(crate::error::BotError::Config(
                "TELEGRAM__TOKEN is required".to_string(),
            ));
        }

        if self.telegram.allowed_user_id == 0 {
            return Err(crate::error::BotError::Config(
                "TELEGRAM__ALLOWED_USER_ID is required".to_string(),
            ));
        }

        if self.opencode.model_provider.is_empty() {
            return Err(crate::error::BotError::Config(
                "OPENCODE__MODEL_PROVIDER is required".to_string(),
            ));
        }

        if self.opencode.model_id.is_empty() {
            return Err(crate::error::BotError::Config(
                "OPENCODE__MODEL_ID is required".to_string(),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_format_mode_from_str() {
        assert!(matches!(
            MessageFormatMode::from_str("raw").unwrap(),
            MessageFormatMode::Raw
        ));
        assert!(matches!(
            MessageFormatMode::from_str("markdown").unwrap(),
            MessageFormatMode::Markdown
        ));
        assert!(MessageFormatMode::from_str("invalid").is_err());
    }

    #[test]
    fn test_default_configs() {
        let bot_config = BotConfig::default();
        assert_eq!(bot_config.sessions_list_limit, 10);
        assert_eq!(bot_config.locale, "en");
        assert!(!bot_config.hide_thinking_messages);

        let server_config = ServerConfig::default();
        assert_eq!(server_config.log_level, "info");

        let file_config = FileConfig::default();
        assert_eq!(file_config.max_file_size_kb, 100);
    }

    #[test]
    fn test_validate_config() {
        let valid_config = AppConfig {
            telegram: TelegramConfig {
                token: "test_token".to_string(),
                allowed_user_id: 123456,
                proxy_url: None,
            },
            opencode: OpencodeConfig {
                api_url: default_opencode_url(),
                username: default_opencode_username(),
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
        };
        assert!(valid_config.validate().is_ok());

        let invalid_config = AppConfig {
            telegram: TelegramConfig {
                token: "".to_string(),
                allowed_user_id: 0,
                proxy_url: None,
            },
            opencode: OpencodeConfig {
                api_url: default_opencode_url(),
                username: default_opencode_username(),
                password: None,
                model_provider: "".to_string(),
                model_id: "".to_string(),
            },
            stt: None,
            tts: None,
            bot: BotConfig::default(),
            server: ServerConfig::default(),
            files: FileConfig::default(),
            open: OpenConfig::default(),
        };
        assert!(invalid_config.validate().is_err());
    }
}

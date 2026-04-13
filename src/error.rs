use thiserror::Error;

#[derive(Error, Debug)]
pub enum BotError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("OpenCode API error: {0}")]
    OpencodeApi(String),

    #[error("Telegram API error: {0}")]
    Telegram(#[from] teloxide::RequestError),

    #[error("HTTP client error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Event source error: {0}")]
    EventSource(String),

    #[error("Scheduled task error: {0}")]
    ScheduledTask(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Unauthorized access attempt from user {0}")]
    Unauthorized(i64),

    #[error("Session not found: {0}")]
    SessionNotFound(String),

    #[error("Project not found: {0}")]
    ProjectNotFound(String),

    #[error("Invalid cron expression: {0}")]
    InvalidCron(String),

    #[error("Task limit exceeded: max {0} tasks allowed")]
    TaskLimitExceeded(usize),

    #[error("Interaction blocked: {0}")]
    InteractionBlocked(String),

    #[error("STT not configured")]
    SttNotConfigured,

    #[error("TTS not configured")]
    TtsNotConfigured,

    #[error("Voice processing error: {0}")]
    VoiceProcessing(String),

    #[error("File too large: {0} KB exceeds limit of {1} KB")]
    FileTooLarge(usize, usize),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

pub type Result<T> = std::result::Result<T, BotError>;

impl From<config::ConfigError> for BotError {
    fn from(err: config::ConfigError) -> Self {
        BotError::Config(err.to_string())
    }
}



impl From<cron::error::Error> for BotError {
    fn from(err: cron::error::Error) -> Self {
        BotError::InvalidCron(err.to_string())
    }
}

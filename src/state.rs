use teloxide::utils::command::BotCommands;

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
        prev_directory: Option<String>,
        model: Option<String>,
    },
    AwaitingModel {
        session_id: Option<String>,
        directory: Option<String>,
    },
    AwaitingSessionSelection {
        prev_session_id: Option<String>,
        prev_directory: Option<String>,
        prev_model: Option<String>,
    },
}

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "snake_case",
    description = "These commands are supported:"
)]
pub enum Command {
    #[command(description = "Display this help text.")]
    Help,
    #[command(description = "Start the bot and initialize a session.")]
    Start,
    #[command(description = "Create a new session.")]
    Session,
    #[command(description = "Set the active project directory.")]
    Project,
    #[command(description = "Set the AI model.")]
    Model,
    #[command(description = "Abort the current generation.")]
    Abort,
    #[command(description = "List all sessions.")]
    ListSessions,
    #[command(description = "Fetch a file from the host machine.", hide)]
    Fetch(String),
}

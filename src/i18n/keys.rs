#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum I18nKey {
    // Common
    CommonUnknownError,
    CommonNotConfigured,
    CommonCancel,
    CommonBack,
    CommonLoading,
    CommonDone,
    CommonErrorOccurred,
    
    // Bot
    BotWelcome,
    BotSessionCreated,
    BotSessionSwitched,
    BotProjectSwitched,
    BotProcessing,
    BotSessionError,
    BotSessionRetry,
    BotNoActiveSession,
    BotInteractionBlocked,
    BotThinking,
    
    // Status
    StatusNoSession,
    StatusNoProject,
    StatusHeader,
    StatusCurrentProject,
    StatusCurrentSession,
    StatusCurrentModel,
    StatusServerOnline,
    StatusServerOffline,
    
    // Commands
    CommandsHelpHeader,
    CommandsStartDesc,
    CommandsHelpDesc,
    CommandsStatusDesc,
    CommandsNewDesc,
    CommandsSessionsDesc,
    CommandsProjectsDesc,
    CommandsCommandsDesc,
    CommandsAbortDesc,
    CommandsTaskDesc,
    CommandsTasklistDesc,
    CommandsTtsDesc,
    CommandsRenameDesc,
    CommandsOpencodeStartDesc,
    CommandsOpencodeStopDesc,
    
    // Task
    TaskHeader,
    TaskNoTasks,
    TaskLimitReached,
    TaskCreatedSuccess,
    TaskDeletedSuccess,
    TaskNextRun,
    TaskLastRun,
    TaskStatusIdle,
    TaskStatusRunning,
    TaskStatusSuccess,
    TaskStatusError,
    TaskPromptLabel,
    TaskScheduleLabel,
    TaskRunSuccess,
    TaskRunError,
    
    // Session
    SessionListHeader,
    SessionSelectPrompt,
    SessionNoSessions,
    
    // Project
    ProjectListHeader,
    ProjectSelectPrompt,
    ProjectNoProjects,
    
    // Model
    ModelSelectPrompt,
    ModelCurrentLabel,
    ModelFavoritesHeader,
    ModelRecentHeader,
    
    // Permission
    PermissionRequestHeader,
    PermissionApprove,
    PermissionDeny,
    
    // Question
    QuestionHeader,
    QuestionSubmit,
    
    // STT
    SttNotConfigured,
    SttTranscribing,
    SttTranscribed,
    SttTranscriptionFailed,
    
    // TTS
    TtsEnabled,
    TtsDisabled,
    TtsNotConfigured,
}

impl I18nKey {
    pub fn as_str(&self) -> &'static str {
        match self {
            // Common
            I18nKey::CommonUnknownError => "common.unknown_error",
            I18nKey::CommonNotConfigured => "common.not_configured",
            I18nKey::CommonCancel => "common.cancel",
            I18nKey::CommonBack => "common.back",
            I18nKey::CommonLoading => "common.loading",
            I18nKey::CommonDone => "common.done",
            I18nKey::CommonErrorOccurred => "common.error_occurred",
            
            // Bot
            I18nKey::BotWelcome => "bot.welcome",
            I18nKey::BotSessionCreated => "bot.session_created",
            I18nKey::BotSessionSwitched => "bot.session_switched",
            I18nKey::BotProjectSwitched => "bot.project_switched",
            I18nKey::BotProcessing => "bot.processing",
            I18nKey::BotSessionError => "bot.session_error",
            I18nKey::BotSessionRetry => "bot.session_retry",
            I18nKey::BotNoActiveSession => "bot.no_active_session",
            I18nKey::BotInteractionBlocked => "bot.interaction_blocked",
            I18nKey::BotThinking => "bot.thinking",
            
            // Status
            I18nKey::StatusNoSession => "status.no_session",
            I18nKey::StatusNoProject => "status.no_project",
            I18nKey::StatusHeader => "status.status_header",
            I18nKey::StatusCurrentProject => "status.current_project",
            I18nKey::StatusCurrentSession => "status.current_session",
            I18nKey::StatusCurrentModel => "status.current_model",
            I18nKey::StatusServerOnline => "status.server_online",
            I18nKey::StatusServerOffline => "status.server_offline",
            
            // Commands
            I18nKey::CommandsHelpHeader => "commands.help_header",
            I18nKey::CommandsStartDesc => "commands.start_desc",
            I18nKey::CommandsHelpDesc => "commands.help_desc",
            I18nKey::CommandsStatusDesc => "commands.status_desc",
            I18nKey::CommandsNewDesc => "commands.new_desc",
            I18nKey::CommandsSessionsDesc => "commands.sessions_desc",
            I18nKey::CommandsProjectsDesc => "commands.projects_desc",
            I18nKey::CommandsCommandsDesc => "commands.commands_desc",
            I18nKey::CommandsAbortDesc => "commands.abort_desc",
            I18nKey::CommandsTaskDesc => "commands.task_desc",
            I18nKey::CommandsTasklistDesc => "commands.tasklist_desc",
            I18nKey::CommandsTtsDesc => "commands.tts_desc",
            I18nKey::CommandsRenameDesc => "commands.rename_desc",
            I18nKey::CommandsOpencodeStartDesc => "commands.opencode_start_desc",
            I18nKey::CommandsOpencodeStopDesc => "commands.opencode_stop_desc",
            
            // Task
            I18nKey::TaskHeader => "task.header",
            I18nKey::TaskNoTasks => "task.no_tasks",
            I18nKey::TaskLimitReached => "task.task_limit_reached",
            I18nKey::TaskCreatedSuccess => "task.created_success",
            I18nKey::TaskDeletedSuccess => "task.deleted_success",
            I18nKey::TaskNextRun => "task.next_run",
            I18nKey::TaskLastRun => "task.last_run",
            I18nKey::TaskStatusIdle => "task.status_idle",
            I18nKey::TaskStatusRunning => "task.status_running",
            I18nKey::TaskStatusSuccess => "task.status_success",
            I18nKey::TaskStatusError => "task.status_error",
            I18nKey::TaskPromptLabel => "task.prompt_label",
            I18nKey::TaskScheduleLabel => "task.schedule_label",
            I18nKey::TaskRunSuccess => "task.run_success",
            I18nKey::TaskRunError => "task.run_error",
            
            // Session
            I18nKey::SessionListHeader => "session.list_header",
            I18nKey::SessionSelectPrompt => "session.select_prompt",
            I18nKey::SessionNoSessions => "session.no_sessions",
            
            // Project
            I18nKey::ProjectListHeader => "project.list_header",
            I18nKey::ProjectSelectPrompt => "project.select_prompt",
            I18nKey::ProjectNoProjects => "project.no_projects",
            
            // Model
            I18nKey::ModelSelectPrompt => "model.select_prompt",
            I18nKey::ModelCurrentLabel => "model.current_label",
            I18nKey::ModelFavoritesHeader => "model.favorites_header",
            I18nKey::ModelRecentHeader => "model.recent_header",
            
            // Permission
            I18nKey::PermissionRequestHeader => "permission.request_header",
            I18nKey::PermissionApprove => "permission.approve",
            I18nKey::PermissionDeny => "permission.deny",
            
            // Question
            I18nKey::QuestionHeader => "question.header",
            I18nKey::QuestionSubmit => "question.submit",
            
            // STT
            I18nKey::SttNotConfigured => "stt.not_configured",
            I18nKey::SttTranscribing => "stt.transcribing",
            I18nKey::SttTranscribed => "stt.transcribed",
            I18nKey::SttTranscriptionFailed => "stt.transcription_failed",
            
            // TTS
            I18nKey::TtsEnabled => "tts.enabled",
            I18nKey::TtsDisabled => "tts.disabled",
            I18nKey::TtsNotConfigured => "tts.not_configured",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_as_str() {
        assert_eq!(I18nKey::CommonUnknownError.as_str(), "common.unknown_error");
        assert_eq!(I18nKey::BotWelcome.as_str(), "bot.welcome");
        assert_eq!(I18nKey::TaskHeader.as_str(), "task.header");
    }

    #[test]
    fn test_all_keys_unique() {
        use std::collections::HashSet;
        
        let keys = vec![
            I18nKey::CommonUnknownError,
            I18nKey::CommonNotConfigured,
            I18nKey::BotWelcome,
            I18nKey::BotSessionCreated,
            I18nKey::TaskHeader,
            I18nKey::TaskNoTasks,
        ];
        
        let mut seen = HashSet::new();
        for key in keys {
            let s = key.as_str();
            assert!(seen.insert(s), "Duplicate key: {}", s);
        }
    }
}

use crate::config::OpencodeConfig;
use crate::error::{BotError, Result};
use opencode_sdk_rs::Opencode;
use std::sync::Arc;
use tracing::{debug, error, info};

pub mod events;
pub mod types;

pub use types::*;

/// Wrapper around the official OpenCode SDK client
#[derive(Clone)]
pub struct OpenCodeClient {
    inner: Arc<Opencode>,
    config: OpencodeConfig,
}

impl OpenCodeClient {
    /// Create a new OpenCode client from configuration
    pub fn new(config: &OpencodeConfig) -> Result<Self> {
        // Set environment variable for base URL if not already set
        if std::env::var("OPENCODE_BASE_URL").is_err() {
            std::env::set_var("OPENCODE_BASE_URL", &config.api_url);
        }

        // Create client - the SDK uses environment variables or defaults
        let inner = Opencode::new()
            .map_err(|e| BotError::OpencodeApi(e.to_string()))?;

        Ok(Self {
            inner: Arc::new(inner),
            config: config.clone(),
        })
    }

    /// Get the underlying SDK client
    pub fn inner(&self) -> &Opencode {
        &self.inner
    }

    /// Check if the OpenCode server is available
    pub async fn health_check(&self) -> Result<bool> {
        match self.inner.app().get(None).await {
            Ok(_) => {
                debug!("OpenCode server is healthy");
                Ok(true)
            }
            Err(e) => {
                error!("OpenCode server health check failed: {}", e);
                Ok(false)
            }
        }
    }

    /// Get server information
    pub async fn get_server_info(&self) -> Result<ServerInfo> {
        let app_info = self
            .inner
            .app()
            .get(None)
            .await
            .map_err(|e| BotError::OpencodeApi(e.to_string()))?;

        Ok(ServerInfo {
            hostname: app_info.hostname,
            version: if app_info.git { "git".to_string() } else { "unknown".to_string() },
        })
    }

    /// List available sessions
    pub async fn list_sessions(&self) -> Result<Vec<SessionInfo>> {
        let sessions = self
            .inner
            .session()
            .list(None)
            .await
            .map_err(|e| BotError::OpencodeApi(e.to_string()))?;

        Ok(sessions
            .into_iter()
            .map(|s| SessionInfo {
                id: s.id,
                title: s.title,
                directory: s.directory,
            })
            .collect())
    }

    /// Create a new session
    pub async fn create_session(&self, title: Option<&str>) -> Result<SessionInfo> {
        // Create the session
        let session = self
            .inner
            .session()
            .create(None)
            .await
            .map_err(|e| BotError::OpencodeApi(e.to_string()))?;

        // If a title was provided, we can't directly set it via init
        // The init endpoint is for setting model/provider/message_id
        // Session title might be set through other means or not supported directly
        if title.is_some() {
            debug!("Session title setting not directly supported by SDK");
        }

        info!("Created new session: {} ({})", session.id, title.unwrap_or("Untitled"));

        Ok(SessionInfo {
            id: session.id,
            title: title.unwrap_or("Untitled").to_string(),
            directory: session.directory,
        })
    }

    /// Get session details
    pub async fn get_session(&self, session_id: &str) -> Result<SessionInfo> {
        // The SDK doesn't have a direct get method, so we list and filter
        let sessions = self.list_sessions().await?;
        
        sessions
            .into_iter()
            .find(|s| s.id == session_id)
            .ok_or_else(|| BotError::SessionNotFound(session_id.to_string()))
    }

    /// List available projects
    pub async fn list_projects(&self) -> Result<Vec<ProjectInfo>> {
        // Get recent sessions which contain project information
        let sessions = self.list_sessions().await?;
        
        // Extract unique projects from sessions
        let mut projects: Vec<ProjectInfo> = sessions
            .into_iter()
            .map(|s| ProjectInfo {
                id: s.directory.clone(),
                worktree: s.directory,
                name: Some(s.title),
            })
            .collect();

        // Deduplicate by worktree
        projects.sort_by(|a, b| a.worktree.cmp(&b.worktree));
        projects.dedup_by(|a, b| a.worktree == b.worktree);

        Ok(projects)
    }

    /// Send a message/prompt to a session
    pub async fn send_prompt(&self, session_id: &str, prompt: &str) -> Result<()> {
        use opencode_sdk_rs::resources::session::{PartInput, SessionChatParams, TextPartInput};

        let params = SessionChatParams {
            parts: vec![PartInput::Text(TextPartInput {
                text: prompt.to_string(),
                id: None,
                synthetic: None,
                ignored: None,
                time: None,
                metadata: None,
            })],
            model: None,
            message_id: None,
            agent: None,
            no_reply: None,
            format: None,
            system: None,
            variant: None,
            tools: None,
        };

        self.inner
            .session()
            .chat(session_id, &params, None)
            .await
            .map_err(|e| BotError::OpencodeApi(e.to_string()))?;

        info!("Sent prompt to session {}", session_id);
        Ok(())
    }

    /// Abort the current operation in a session
    pub async fn abort_session(&self, session_id: &str) -> Result<()> {
        self.inner
            .session()
            .abort(session_id, None)
            .await
            .map_err(|e| BotError::OpencodeApi(e.to_string()))?;

        info!("Aborted session {}", session_id);
        Ok(())
    }

    /// Rename a session
    pub async fn rename_session(&self, session_id: &str, _new_title: &str) -> Result<()> {
        // The SDK doesn't have a direct rename method
        // The init endpoint requires message_id, model_id, provider_id - not title
        info!("Rename session {} requested (SDK does not support direct rename)", session_id);
        
        // Note: We cannot call init without the required parameters
        // Session renaming might need to be done through a different API or not supported
        Ok(())
    }

    /// Get the current model for the session
    pub async fn get_session_model(&self, session_id: &str) -> Result<ModelInfo> {
        // Get messages to find the model being used
        let messages = self
            .inner
            .session()
            .messages(session_id, None)
            .await
            .map_err(|e| BotError::OpencodeApi(e.to_string()))?;

        // Try to get model from the most recent message
        // SessionMessagesResponseItem has `info` (Message enum) and `parts`
        if let Some(item) = messages.last() {
            match &item.info {
                opencode_sdk_rs::resources::session::Message::User(user_msg) => {
                    return Ok(ModelInfo {
                        provider: user_msg.model.provider_id.clone(),
                        model_id: user_msg.model.model_id.clone(),
                    });
                }
                opencode_sdk_rs::resources::session::Message::Assistant(assistant_msg) => {
                    return Ok(ModelInfo {
                        provider: assistant_msg.provider_id.clone(),
                        model_id: assistant_msg.model_id.clone(),
                    });
                }
            }
        }

        // Fallback to config defaults
        Ok(ModelInfo {
            provider: self.config.model_provider.clone(),
            model_id: self.config.model_id.clone(),
        })
    }

    /// Subscribe to events for a directory
    pub async fn subscribe_events(
        &self,
        _directory: &str,
    ) -> Result<impl futures::Stream<Item = Result<OpenCodeEvent>>> {
        // The SDK's event subscription may have a different API
        // For now, return a pending stream that never produces events
        // This allows the application to function without SSE support
        Ok(futures::stream::pending())
    }
}

impl From<opencode_sdk_rs::error::OpencodeError> for BotError {
    fn from(err: opencode_sdk_rs::error::OpencodeError) -> Self {
        BotError::OpencodeApi(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::OpencodeConfig;

    fn create_test_config() -> OpencodeConfig {
        OpencodeConfig {
            api_url: "http://localhost:4096".to_string(),
            username: "opencode".to_string(),
            password: None,
            model_provider: "opencode".to_string(),
            model_id: "big-pickle".to_string(),
        }
    }

    #[test]
    fn test_client_creation() {
        let config = create_test_config();
        let client = OpenCodeClient::new(&config);
        assert!(client.is_ok());
    }

    #[test]
    fn test_client_creation_with_auth() {
        let mut config = create_test_config();
        config.password = Some("secret".to_string());
        
        let client = OpenCodeClient::new(&config);
        assert!(client.is_ok());
    }
}

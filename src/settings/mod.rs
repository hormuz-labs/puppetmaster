use crate::error::Result;
use crate::opencode::types::{ModelInfo, ProjectInfo, SessionInfo};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite};
use std::path::Path;
use std::sync::Arc;
use tracing::{debug, info};

pub mod db;

pub use db::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Settings {
    pub current_project: Option<ProjectInfo>,
    pub current_session: Option<SessionInfo>,
    pub current_agent: Option<String>,
    pub current_model: Option<ModelInfo>,
    pub pinned_message_id: Option<i64>,
    pub tts_enabled: bool,
}

/// Manages persistent settings storage
#[derive(Clone)]
pub struct SettingsManager {
    db: Pool<Sqlite>,
    cache: Arc<parking_lot::RwLock<Settings>>,
}

impl SettingsManager {
    /// Initialize the settings manager with a database pool
    pub async fn new(db_path: impl AsRef<Path>) -> Result<Self> {
        let db = init_database(db_path).await?;
        
        let manager = Self {
            db,
            cache: Arc::new(parking_lot::RwLock::new(Settings::default())),
        };

        // Load settings from database into cache
        manager.load_from_db().await?;
        
        info!("Settings manager initialized");
        Ok(manager)
    }

    /// Get current project
    pub fn current_project(&self) -> Option<ProjectInfo> {
        self.cache.read().current_project.clone()
    }

    /// Set current project
    pub async fn set_current_project(&self, project: ProjectInfo) -> Result<()> {
        {
            let mut cache = self.cache.write();
            cache.current_project = Some(project.clone());
        }
        
        save_setting(&self.db, "current_project", &project).await?;
        debug!("Set current project: {:?}", project.worktree);
        Ok(())
    }

    /// Clear current project
    pub async fn clear_project(&self) -> Result<()> {
        {
            let mut cache = self.cache.write();
            cache.current_project = None;
        }
        
        delete_setting(&self.db, "current_project").await?;
        Ok(())
    }

    /// Get current session
    pub fn current_session(&self) -> Option<SessionInfo> {
        self.cache.read().current_session.clone()
    }

    /// Set current session
    pub async fn set_current_session(&self, session: SessionInfo) -> Result<()> {
        {
            let mut cache = self.cache.write();
            cache.current_session = Some(session.clone());
        }
        
        save_setting(&self.db, "current_session", &session).await?;
        debug!("Set current session: {:?}", session.id);
        Ok(())
    }

    /// Clear current session
    pub async fn clear_session(&self) -> Result<()> {
        {
            let mut cache = self.cache.write();
            cache.current_session = None;
        }
        
        delete_setting(&self.db, "current_session").await?;
        Ok(())
    }

    /// Get current agent
    pub fn current_agent(&self) -> Option<String> {
        self.cache.read().current_agent.clone()
    }

    /// Set current agent
    pub async fn set_current_agent(&self, agent: String) -> Result<()> {
        {
            let mut cache = self.cache.write();
            cache.current_agent = Some(agent.clone());
        }
        
        save_setting(&self.db, "current_agent", &agent).await?;
        Ok(())
    }

    /// Clear current agent
    pub async fn clear_agent(&self) -> Result<()> {
        {
            let mut cache = self.cache.write();
            cache.current_agent = None;
        }
        
        delete_setting(&self.db, "current_agent").await?;
        Ok(())
    }

    /// Get current model
    pub fn current_model(&self) -> Option<ModelInfo> {
        self.cache.read().current_model.clone()
    }

    /// Set current model
    pub async fn set_current_model(&self, model: ModelInfo) -> Result<()> {
        {
            let mut cache = self.cache.write();
            cache.current_model = Some(model.clone());
        }
        
        save_setting(&self.db, "current_model", &model).await?;
        debug!("Set current model: {:?}", model);
        Ok(())
    }

    /// Clear current model
    pub async fn clear_model(&self) -> Result<()> {
        {
            let mut cache = self.cache.write();
            cache.current_model = None;
        }
        
        delete_setting(&self.db, "current_model").await?;
        Ok(())
    }

    /// Get pinned message ID
    pub fn pinned_message_id(&self) -> Option<i64> {
        self.cache.read().pinned_message_id
    }

    /// Set pinned message ID
    pub async fn set_pinned_message_id(&self, message_id: i64) -> Result<()> {
        {
            let mut cache = self.cache.write();
            cache.pinned_message_id = Some(message_id);
        }
        
        save_setting(&self.db, "pinned_message_id", &message_id).await?;
        Ok(())
    }

    /// Clear pinned message ID
    pub async fn clear_pinned_message_id(&self) -> Result<()> {
        {
            let mut cache = self.cache.write();
            cache.pinned_message_id = None;
        }
        
        delete_setting(&self.db, "pinned_message_id").await?;
        Ok(())
    }

    /// Get TTS enabled status
    pub fn tts_enabled(&self) -> bool {
        self.cache.read().tts_enabled
    }

    /// Set TTS enabled status
    pub async fn set_tts_enabled(&self, enabled: bool) -> Result<()> {
        {
            let mut cache = self.cache.write();
            cache.tts_enabled = enabled;
        }
        
        save_setting(&self.db, "tts_enabled", &enabled).await?;
        debug!("Set TTS enabled: {}", enabled);
        Ok(())
    }

    /// Get all settings
    pub fn get_all(&self) -> Settings {
        self.cache.read().clone()
    }

    /// Get the database pool
    pub fn get_db_pool(&self) -> Pool<Sqlite> {
        self.db.clone()
    }

    /// Load settings from database into cache
    async fn load_from_db(&self) -> Result<()> {
        let mut cache = self.cache.write();

        // Load each setting
        if let Ok(Some(value)) = load_setting::<ProjectInfo>(&self.db, "current_project").await {
            cache.current_project = Some(value);
        }

        if let Ok(Some(value)) = load_setting::<SessionInfo>(&self.db, "current_session").await {
            cache.current_session = Some(value);
        }

        if let Ok(Some(value)) = load_setting::<String>(&self.db, "current_agent").await {
            cache.current_agent = Some(value);
        }

        if let Ok(Some(value)) = load_setting::<ModelInfo>(&self.db, "current_model").await {
            cache.current_model = Some(value);
        }

        if let Ok(Some(value)) = load_setting::<i64>(&self.db, "pinned_message_id").await {
            cache.pinned_message_id = Some(value);
        }

        if let Ok(Some(value)) = load_setting::<bool>(&self.db, "tts_enabled").await {
            cache.tts_enabled = value;
        }

        info!("Settings loaded from database");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::db::run_migrations;
    use sqlx::sqlite::SqlitePoolOptions;
    use std::sync::Arc;
    use parking_lot::RwLock;

    async fn create_test_manager() -> SettingsManager {
        // Use in-memory database for tests
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        
        // Run migrations
        run_migrations(&pool).await.ok();
        
        // Create manager with the pool
        let manager = SettingsManager {
            db: pool,
            cache: Arc::new(parking_lot::RwLock::new(Settings::default())),
        };
        
        manager
    }

    #[tokio::test]
    async fn test_project_settings() {
        let manager = create_test_manager().await;

        // Initially no project
        assert!(manager.current_project().is_none());

        // Set project
        let project = ProjectInfo {
            id: "test-id".to_string(),
            worktree: "/test/path".to_string(),
            name: Some("Test Project".to_string()),
        };
        manager.set_current_project(project.clone()).await.unwrap();

        // Verify
        let current = manager.current_project().unwrap();
        assert_eq!(current.id, "test-id");
        assert_eq!(current.worktree, "/test/path");

        // Clear
        manager.clear_project().await.unwrap();
        assert!(manager.current_project().is_none());
    }

    #[tokio::test]
    async fn test_session_settings() {
        let manager = create_test_manager().await;

        let session = SessionInfo {
            id: "session-123".to_string(),
            title: "Test Session".to_string(),
            directory: "/test".to_string(),
        };
        manager.set_current_session(session.clone()).await.unwrap();

        let current = manager.current_session().unwrap();
        assert_eq!(current.id, "session-123");
        assert_eq!(current.title, "Test Session");
    }

    #[tokio::test]
    async fn test_tts_settings() {
        let manager = create_test_manager().await;

        // Default is false
        assert!(!manager.tts_enabled());

        // Enable TTS
        manager.set_tts_enabled(true).await.unwrap();
        assert!(manager.tts_enabled());

        // Disable TTS
        manager.set_tts_enabled(false).await.unwrap();
        assert!(!manager.tts_enabled());
    }

    #[tokio::test]
    async fn test_model_settings() {
        let manager = create_test_manager().await;

        let model = ModelInfo {
            provider: "opencode".to_string(),
            model_id: "big-pickle".to_string(),
        };
        manager.set_current_model(model.clone()).await.unwrap();

        let current = manager.current_model().unwrap();
        assert_eq!(current.provider, "opencode");
        assert_eq!(current.model_id, "big-pickle");
    }
}

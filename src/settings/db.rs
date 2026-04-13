use crate::error::Result;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};
use std::path::Path;
use tracing::info;

/// Initialize the SQLite database
pub async fn init_database(db_path: impl AsRef<Path>) -> Result<Pool<Sqlite>> {
    let db_path = db_path.as_ref();
    
    // Ensure parent directory exists
    if let Some(parent) = db_path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    let db_url = format!("sqlite://{}", db_path.display());
    
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;

    // Run migrations manually
    run_migrations(&pool).await?;

    info!("Database initialized at {:?}", db_path);
    Ok(pool)
}

/// Run database migrations
pub async fn run_migrations(db: &Pool<Sqlite>) -> crate::error::Result<()> {
    // Create tables if they don't exist
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY NOT NULL,
            value TEXT NOT NULL,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(db)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS scheduled_tasks (
            id TEXT PRIMARY KEY NOT NULL,
            kind TEXT NOT NULL,
            prompt TEXT NOT NULL,
            schedule_summary TEXT,
            run_at TEXT,
            cron_expr TEXT,
            next_run_at DATETIME,
            last_run_at DATETIME,
            last_status TEXT DEFAULT 'idle',
            last_error TEXT,
            run_count INTEGER DEFAULT 0,
            model_provider TEXT NOT NULL,
            model_id TEXT NOT NULL,
            project_worktree TEXT,
            session_id TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(db)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS session_cache (
            id TEXT PRIMARY KEY NOT NULL,
            directory TEXT NOT NULL,
            worktree TEXT NOT NULL,
            title TEXT,
            last_updated INTEGER NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(db)
    .await?;

    // Create indexes
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_session_cache_worktree ON session_cache(worktree)")
        .execute(db)
        .await?;
    
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_scheduled_tasks_next_run ON scheduled_tasks(next_run_at)")
        .execute(db)
        .await?;

    Ok(())
}

/// Save a setting to the database
pub async fn save_setting<T: Serialize>(
    db: &Pool<Sqlite>,
    key: &str,
    value: &T,
) -> Result<()> {
    let json_value = serde_json::to_string(value)?;
    
    sqlx::query(
        r#"
        INSERT INTO settings (key, value, updated_at)
        VALUES (?1, ?2, CURRENT_TIMESTAMP)
        ON CONFLICT(key) DO UPDATE SET
            value = excluded.value,
            updated_at = excluded.updated_at
        "#,
    )
    .bind(key)
    .bind(json_value)
    .execute(db)
    .await?;

    Ok(())
}

/// Load a setting from the database
pub async fn load_setting<T: DeserializeOwned>(
    db: &Pool<Sqlite>,
    key: &str,
) -> Result<Option<T>> {
    let row: Option<(String,)> = sqlx::query_as(
        r#"
        SELECT value FROM settings WHERE key = ?1
        "#,
    )
    .bind(key)
    .fetch_optional(db)
    .await?;

    match row {
        Some((json_value,)) => {
            let value = serde_json::from_str(&json_value)?;
            Ok(Some(value))
        }
        None => Ok(None),
    }
}

/// Delete a setting from the database
pub async fn delete_setting(db: &Pool<Sqlite>, key: &str) -> Result<()> {
    sqlx::query("DELETE FROM settings WHERE key = ?1")
        .bind(key)
        .execute(db)
        .await?;

    Ok(())
}

/// Get all settings as a map
pub async fn list_settings(db: &Pool<Sqlite>) -> Result<Vec<(String, String)>> {
    let rows: Vec<(String, String)> = sqlx::query_as(
        r#"
        SELECT key, value FROM settings ORDER BY key
        "#,
    )
    .fetch_all(db)
    .await?;

    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn create_test_db() -> Pool<Sqlite> {
        // Use in-memory database for tests
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        
        // Run migrations
        run_migrations(&pool).await.unwrap();
        
        pool
    }

    #[tokio::test]
    async fn test_save_and_load_setting() {
        let db = create_test_db().await;

        // Save a string
        save_setting(&db, "test_key", &"test_value").await.unwrap();

        // Load it back
        let loaded: Option<String> = load_setting(&db, "test_key").await.unwrap();
        assert_eq!(loaded, Some("test_value".to_string()));

        // Load non-existent key
        let not_found: Option<String> = load_setting(&db, "non_existent").await.unwrap();
        assert_eq!(not_found, None);
    }

    #[tokio::test]
    async fn test_save_struct() {
        let db = create_test_db().await;

        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        struct TestStruct {
            name: String,
            count: i32,
        }

        let original = TestStruct {
            name: "test".to_string(),
            count: 42,
        };

        save_setting(&db, "struct_key", &original).await.unwrap();

        let loaded: Option<TestStruct> = load_setting(&db, "struct_key").await.unwrap();
        assert_eq!(loaded, Some(original));
    }

    #[tokio::test]
    async fn test_update_setting() {
        let db = create_test_db().await;

        save_setting(&db, "key", &"first").await.unwrap();
        save_setting(&db, "key", &"second").await.unwrap();

        let loaded: Option<String> = load_setting(&db, "key").await.unwrap();
        assert_eq!(loaded, Some("second".to_string()));
    }

    #[tokio::test]
    async fn test_delete_setting() {
        let db = create_test_db().await;

        save_setting(&db, "to_delete", &"value").await.unwrap();
        
        let exists: Option<String> = load_setting(&db, "to_delete").await.unwrap();
        assert!(exists.is_some());

        delete_setting(&db, "to_delete").await.unwrap();

        let deleted: Option<String> = load_setting(&db, "to_delete").await.unwrap();
        assert!(deleted.is_none());
    }
}

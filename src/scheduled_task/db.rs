use super::{ScheduledTask, TaskStatus};
use crate::error::Result;
use chrono::{DateTime, Utc};
use sqlx::{Pool, Sqlite};

/// Save a scheduled task to the database
pub async fn save_scheduled_task(db: &Pool<Sqlite>, task: &ScheduledTask) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO scheduled_tasks (
            id, kind, prompt, schedule_summary, run_at, cron_expr,
            next_run_at, last_run_at, last_status, last_error, run_count,
            model_provider, model_id, project_worktree, session_id, created_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)
        ON CONFLICT(id) DO UPDATE SET
            kind = excluded.kind,
            prompt = excluded.prompt,
            schedule_summary = excluded.schedule_summary,
            run_at = excluded.run_at,
            cron_expr = excluded.cron_expr,
            next_run_at = excluded.next_run_at,
            last_run_at = excluded.last_run_at,
            last_status = excluded.last_status,
            last_error = excluded.last_error,
            run_count = excluded.run_count,
            model_provider = excluded.model_provider,
            model_id = excluded.model_id,
            project_worktree = excluded.project_worktree,
            session_id = excluded.session_id
        "#,
    )
    .bind(&task.id)
    .bind(match task.kind {
        super::TaskKind::Once => "once",
        super::TaskKind::Cron => "cron",
    })
    .bind(&task.prompt)
    .bind(&task.schedule_summary)
    .bind(task.run_at.map(|d| d.to_rfc3339()))
    .bind(&task.cron_expr)
    .bind(task.next_run_at.map(|d| d.to_rfc3339()))
    .bind(task.last_run_at.map(|d| d.to_rfc3339()))
    .bind(match task.last_status {
        TaskStatus::Idle => "idle",
        TaskStatus::Running => "running",
        TaskStatus::Success => "success",
        TaskStatus::Error => "error",
    })
    .bind(&task.last_error)
    .bind(task.run_count)
    .bind(&task.model_provider)
    .bind(&task.model_id)
    .bind(&task.project_worktree)
    .bind(&task.session_id)
    .bind(task.created_at.to_rfc3339())
    .execute(db)
    .await?;

    Ok(())
}

/// Load a scheduled task by ID
pub async fn load_scheduled_task(db: &Pool<Sqlite>, task_id: &str) -> Result<Option<ScheduledTask>> {
    let row = sqlx::query(
        r#"
        SELECT * FROM scheduled_tasks WHERE id = ?1
        "#,
    )
    .bind(task_id)
    .fetch_optional(db)
    .await?;

    match row {
        Some(r) => Ok(Some(row_to_task(r)?)),
        None => Ok(None),
    }
}

/// List all scheduled tasks
pub async fn list_scheduled_tasks(db: &Pool<Sqlite>) -> Result<Vec<ScheduledTask>> {
    let rows = sqlx::query(
        r#"
        SELECT * FROM scheduled_tasks ORDER BY created_at DESC
        "#,
    )
    .fetch_all(db)
    .await?;

    rows.into_iter()
        .map(row_to_task)
        .collect::<Result<Vec<_>>>()
}

/// Count scheduled tasks
pub async fn count_scheduled_tasks(db: &Pool<Sqlite>) -> Result<usize> {
    let count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*) FROM scheduled_tasks
        "#,
    )
    .fetch_one(db)
    .await?;

    Ok(count as usize)
}

/// Delete a scheduled task
pub async fn delete_scheduled_task(db: &Pool<Sqlite>, task_id: &str) -> Result<()> {
    sqlx::query("DELETE FROM scheduled_tasks WHERE id = ?1")
        .bind(task_id)
        .execute(db)
        .await?;

    Ok(())
}

/// Update task status
pub async fn update_task_status(
    db: &Pool<Sqlite>,
    task_id: &str,
    status: TaskStatus,
    error: Option<&str>,
) -> Result<()> {
    let now = Utc::now();
    
    sqlx::query(
        r#"
        UPDATE scheduled_tasks
        SET last_status = ?1,
            last_error = ?2,
            last_run_at = CASE WHEN ?1 IN ('success', 'error') THEN ?3 ELSE last_run_at END,
            run_count = CASE WHEN ?1 IN ('success', 'error') THEN run_count + 1 ELSE run_count END
        WHERE id = ?4
        "#,
    )
    .bind(match status {
        TaskStatus::Idle => "idle",
        TaskStatus::Running => "running",
        TaskStatus::Success => "success",
        TaskStatus::Error => "error",
    })
    .bind(error)
    .bind(now.to_rfc3339())
    .bind(task_id)
    .execute(db)
    .await?;

    Ok(())
}

/// Update next run time
pub async fn update_next_run(
    db: &Pool<Sqlite>,
    task_id: &str,
    next_run: Option<DateTime<Utc>>,
) -> Result<()> {
    sqlx::query(
        r#"
        UPDATE scheduled_tasks
        SET next_run_at = ?1
        WHERE id = ?2
        "#,
    )
    .bind(next_run.map(|d| d.to_rfc3339()))
    .bind(task_id)
    .execute(db)
    .await?;

    Ok(())
}

/// Convert a SQL row to a ScheduledTask
fn row_to_task(row: sqlx::sqlite::SqliteRow) -> Result<ScheduledTask> {
    use sqlx::Row;
    
    let kind: String = row.try_get("kind")?;
    let last_status: String = row.try_get("last_status")?;
    
    Ok(ScheduledTask {
        id: row.try_get("id")?,
        kind: match kind.as_str() {
            "once" => super::TaskKind::Once,
            "cron" => super::TaskKind::Cron,
            _ => super::TaskKind::Once,
        },
        prompt: row.try_get("prompt")?,
        schedule_summary: row.try_get("schedule_summary")?,
        run_at: row.try_get::<Option<String>, _>("run_at")?.and_then(|d| DateTime::parse_from_rfc3339(&d).ok().map(|d| d.with_timezone(&Utc))),
        cron_expr: row.try_get("cron_expr")?,
        next_run_at: row.try_get::<Option<String>, _>("next_run_at")?.and_then(|d| DateTime::parse_from_rfc3339(&d).ok().map(|d| d.with_timezone(&Utc))),
        last_run_at: row.try_get::<Option<String>, _>("last_run_at")?.and_then(|d| DateTime::parse_from_rfc3339(&d).ok().map(|d| d.with_timezone(&Utc))),
        last_status: match last_status.as_str() {
            "idle" => TaskStatus::Idle,
            "running" => TaskStatus::Running,
            "success" => TaskStatus::Success,
            "error" => TaskStatus::Error,
            _ => TaskStatus::Idle,
        },
        last_error: row.try_get("last_error")?,
        run_count: row.try_get("run_count")?,
        model_provider: row.try_get("model_provider")?,
        model_id: row.try_get("model_id")?,
        project_worktree: row.try_get("project_worktree")?,
        session_id: row.try_get("session_id")?,
        created_at: row.try_get::<String, _>("created_at")
            .and_then(|d| DateTime::parse_from_rfc3339(&d).map_err(|e| sqlx::Error::Decode(Box::new(e))))?
            .with_timezone(&Utc),
    })
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::settings::db::run_migrations;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn create_test_db() -> Pool<Sqlite> {
        // Use in-memory database for tests
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        
        // Run migrations
        run_migrations(&pool).await.ok();
        
        pool
    }

    #[tokio::test]
    async fn test_save_and_load_task() {
        let db = create_test_db().await;

        let task = ScheduledTask::new_once(
            "Test prompt".to_string(),
            Utc::now() + chrono::Duration::hours(1),
            "opencode".to_string(),
            "big-pickle".to_string(),
        ).unwrap();

        save_scheduled_task(&db, &task).await.unwrap();

        let loaded = load_scheduled_task(&db, &task.id).await.unwrap();
        assert!(loaded.is_some());
        
        let loaded = loaded.unwrap();
        assert_eq!(loaded.id, task.id);
        assert_eq!(loaded.prompt, task.prompt);
        assert_eq!(loaded.kind, TaskKind::Once);
    }

    #[tokio::test]
    async fn test_list_tasks() {
        let db = create_test_db().await;

        let task1 = ScheduledTask::new_once(
            "Task 1".to_string(),
            Utc::now() + chrono::Duration::hours(1),
            "opencode".to_string(),
            "big-pickle".to_string(),
        ).unwrap();

        let task2 = ScheduledTask::new_cron(
            "Task 2".to_string(),
            "0 0 * * * *".to_string(),
            "opencode".to_string(),
            "big-pickle".to_string(),
        ).unwrap();

        save_scheduled_task(&db, &task1).await.unwrap();
        save_scheduled_task(&db, &task2).await.unwrap();

        let tasks = list_scheduled_tasks(&db).await.unwrap();
        assert_eq!(tasks.len(), 2);
    }

    #[tokio::test]
    async fn test_delete_task() {
        let db = create_test_db().await;

        let task = ScheduledTask::new_once(
            "To delete".to_string(),
            Utc::now() + chrono::Duration::hours(1),
            "opencode".to_string(),
            "big-pickle".to_string(),
        ).unwrap();

        save_scheduled_task(&db, &task).await.unwrap();
        
        let before = count_scheduled_tasks(&db).await.unwrap();
        assert_eq!(before, 1);

        delete_scheduled_task(&db, &task.id).await.unwrap();
        
        let after = count_scheduled_tasks(&db).await.unwrap();
        assert_eq!(after, 0);
    }

    #[tokio::test]
    async fn test_update_status() {
        let db = create_test_db().await;

        let task = ScheduledTask::new_once(
            "Test".to_string(),
            Utc::now() + chrono::Duration::hours(1),
            "opencode".to_string(),
            "big-pickle".to_string(),
        ).unwrap();

        save_scheduled_task(&db, &task).await.unwrap();

        update_task_status(&db, &task.id, TaskStatus::Running, None).await.unwrap();
        
        let loaded = load_scheduled_task(&db, &task.id).await.unwrap().unwrap();
        assert_eq!(loaded.last_status, TaskStatus::Running);

        update_task_status(&db, &task.id, TaskStatus::Error, Some("Test error")).await.unwrap();
        
        let loaded = load_scheduled_task(&db, &task.id).await.unwrap().unwrap();
        assert_eq!(loaded.last_status, TaskStatus::Error);
        assert_eq!(loaded.last_error, Some("Test error".to_string()));
        assert_eq!(loaded.run_count, 1);
    }
}

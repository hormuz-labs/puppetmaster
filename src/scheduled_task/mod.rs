use crate::config::BotConfig;
use crate::error::{BotError, Result};
use chrono::{DateTime, Utc};
use cron::Schedule;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite};
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::{debug, info, warn};
use uuid::Uuid;

pub mod db;

pub use db::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TaskKind {
    Once,
    Cron,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    Idle,
    Running,
    Success,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledTask {
    pub id: String,
    pub kind: TaskKind,
    pub prompt: String,
    pub schedule_summary: String,
    pub run_at: Option<DateTime<Utc>>, // For one-time tasks
    pub cron_expr: Option<String>,     // For cron tasks
    pub next_run_at: Option<DateTime<Utc>>,
    pub last_run_at: Option<DateTime<Utc>>,
    pub last_status: TaskStatus,
    pub last_error: Option<String>,
    pub run_count: i32,
    pub model_provider: String,
    pub model_id: String,
    pub project_worktree: Option<String>,
    pub session_id: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl ScheduledTask {
    /// Create a new one-time task
    pub fn new_once(
        prompt: String,
        run_at: DateTime<Utc>,
        model_provider: String,
        model_id: String,
    ) -> Result<Self> {
        Ok(Self {
            id: Uuid::new_v4().to_string(),
            kind: TaskKind::Once,
            prompt,
            schedule_summary: format!("Once at {}", run_at.format("%Y-%m-%d %H:%M")),
            run_at: Some(run_at),
            cron_expr: None,
            next_run_at: Some(run_at),
            last_run_at: None,
            last_status: TaskStatus::Idle,
            last_error: None,
            run_count: 0,
            model_provider,
            model_id,
            project_worktree: None,
            session_id: None,
            created_at: Utc::now(),
        })
    }

    /// Create a new cron task
    pub fn new_cron(
        prompt: String,
        cron_expr: String,
        model_provider: String,
        model_id: String,
    ) -> Result<Self> {
        // Validate cron expression
        let schedule = Schedule::from_str(&cron_expr)?;
        
        // Calculate next run time
        let next_run = schedule.upcoming(Utc).next();
        
        Ok(Self {
            id: Uuid::new_v4().to_string(),
            kind: TaskKind::Cron,
            prompt,
            schedule_summary: cron_expr.clone(),
            run_at: None,
            cron_expr: Some(cron_expr),
            next_run_at: next_run,
            last_run_at: None,
            last_status: TaskStatus::Idle,
            last_error: None,
            run_count: 0,
            model_provider,
            model_id,
            project_worktree: None,
            session_id: None,
            created_at: Utc::now(),
        })
    }

    /// Compute next run time for cron tasks
    pub fn compute_next_run(&self, after: DateTime<Utc>) -> Option<DateTime<Utc>> {
        match &self.cron_expr {
            Some(expr) => {
                let schedule = Schedule::from_str(expr).ok()?;
                schedule.after(&after).next()
            }
            None => None,
        }
    }

    /// Check if task is due to run
    pub fn is_due(&self) -> bool {
        match self.next_run_at {
            Some(next) => Utc::now() >= next,
            None => false,
        }
    }
}

/// Manages scheduled tasks
#[derive(Clone)]
pub struct TaskScheduler {
    scheduler: Arc<RwLock<JobScheduler>>,
    db: Pool<Sqlite>,
    config: BotConfig,
    tasks: Arc<RwLock<Vec<ScheduledTask>>>,
}

impl TaskScheduler {
    pub async fn new(
        db: Pool<Sqlite>,
        config: BotConfig,
    ) -> Result<Self> {
        let scheduler = JobScheduler::new().await
            .map_err(|e| BotError::ScheduledTask(e.to_string()))?;

        Ok(Self {
            scheduler: Arc::new(RwLock::new(scheduler)),
            db,
            config,
            tasks: Arc::new(RwLock::new(Vec::new())),
        })
    }

    /// Initialize the scheduler and recover existing tasks
    pub async fn initialize(&self) -> Result<()> {
        // Load existing tasks from database
        let tasks = list_scheduled_tasks(&self.db).await?;
        let task_count = tasks.len();
        
        {
            let mut guard = self.tasks.write().await;
            *guard = tasks;
        }

        // Schedule each task
        let tasks_guard = self.tasks.read().await;
        for task in tasks_guard.iter() {
            if let Err(e) = self.schedule_task(task).await {
                warn!("Failed to schedule task {}: {}", task.id, e);
            }
        }
        drop(tasks_guard);

        // Start the scheduler
        let scheduler = self.scheduler.write().await;
        scheduler.start().await
            .map_err(|e| BotError::ScheduledTask(e.to_string()))?;

        info!("Task scheduler initialized with {} tasks", task_count);
        Ok(())
    }

    /// Add a new task
    pub async fn add_task(&self, task: ScheduledTask) -> Result<String> {
        // Check task limit
        let current_count = count_scheduled_tasks(&self.db).await?;
        if current_count >= self.config.task_limit {
            return Err(BotError::TaskLimitExceeded(self.config.task_limit));
        }

        // Save to database
        save_scheduled_task(&self.db, &task).await?;

        // Add to in-memory list
        {
            let mut guard = self.tasks.write().await;
            guard.push(task.clone());
        }

        // Schedule the task
        self.schedule_task(&task).await?;

        info!("Added scheduled task: {}", task.id);
        Ok(task.id)
    }

    /// Remove a task
    pub async fn remove_task(&self, task_id: &str) -> Result<()> {
        // Remove from database
        delete_scheduled_task(&self.db, task_id).await?;

        // Remove from in-memory list
        {
            let mut guard = self.tasks.write().await;
            guard.retain(|t| t.id != task_id);
        }

        info!("Removed scheduled task: {}", task_id);
        Ok(())
    }

    /// Get all tasks
    pub async fn list_tasks(&self) -> Vec<ScheduledTask> {
        self.tasks.read().await.clone()
    }

    /// Get a specific task
    pub async fn get_task(&self, task_id: &str) -> Option<ScheduledTask> {
        self.tasks.read().await.iter()
            .find(|t| t.id == task_id)
            .cloned()
    }

    /// Schedule a task in the job scheduler
    async fn schedule_task(&self, task: &ScheduledTask) -> Result<()> {
        let task_id = task.id.clone();
        
        match task.kind {
            TaskKind::Once => {
                if let Some(run_at) = task.run_at {
                    let now = Utc::now();
                    if run_at > now {
                        let duration = (run_at - now).to_std()
                            .map_err(|_| BotError::ScheduledTask("Invalid duration".to_string()))?;
                        
                        debug!("Scheduled one-time task {} for {:?}", task_id, duration);
                    }
                }
            }
            TaskKind::Cron => {
                if let Some(ref cron_expr) = task.cron_expr {
                    let job_id = task_id.clone();
                    let job = Job::new_async(cron_expr, move |_uuid, _lock| {
                        let id = job_id.clone();
                        Box::pin(async move {
                            info!("Executing cron task: {}", id);
                        })
                    }).map_err(|e| BotError::InvalidCron(e.to_string()))?;

                    let scheduler = self.scheduler.write().await;
                    scheduler.add(job).await
                        .map_err(|e| BotError::ScheduledTask(e.to_string()))?;
                }
            }
        }

        Ok(())
    }

    /// Shutdown the scheduler
    pub async fn shutdown(&self) -> Result<()> {
        let mut scheduler = self.scheduler.write().await;
        scheduler.shutdown().await
            .map_err(|e| BotError::ScheduledTask(e.to_string()))?;
        info!("Task scheduler shutdown");
        Ok(())
    }
}

/// Parse a schedule expression (cron or natural language)
pub fn parse_schedule(expr: &str) -> Result<(TaskKind, String, String)> {
    // Try parsing as cron expression first
    if let Ok(_schedule) = Schedule::from_str(expr) {
        // Valid cron
        let summary = expr.to_string();
        return Ok((TaskKind::Cron, expr.to_string(), summary));
    }

    // Try parsing as English using tokio-cron-scheduler's English feature
    // For now, return an error
    Err(BotError::InvalidCron(format!(
        "Could not parse schedule expression: {}",
        expr
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scheduled_task_new_once() {
        let run_at = Utc::now() + chrono::Duration::hours(1);
        let task = ScheduledTask::new_once(
            "Test prompt".to_string(),
            run_at,
            "opencode".to_string(),
            "big-pickle".to_string(),
        ).unwrap();

        assert_eq!(task.kind, TaskKind::Once);
        assert_eq!(task.prompt, "Test prompt");
        assert!(task.cron_expr.is_none());
        assert!(task.run_at.is_some());
    }

    #[test]
    fn test_scheduled_task_new_cron() {
        let task = ScheduledTask::new_cron(
            "Test prompt".to_string(),
            "0 0 * * * *".to_string(), // Every hour
            "opencode".to_string(),
            "big-pickle".to_string(),
        ).unwrap();

        assert_eq!(task.kind, TaskKind::Cron);
        assert_eq!(task.cron_expr, Some("0 0 * * * *".to_string()));
        assert!(task.next_run_at.is_some());
    }

    #[test]
    fn test_invalid_cron() {
        let result = ScheduledTask::new_cron(
            "Test".to_string(),
            "invalid cron".to_string(),
            "opencode".to_string(),
            "big-pickle".to_string(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_task_is_due() {
        let past = Utc::now() - chrono::Duration::minutes(1);
        let task = ScheduledTask::new_once(
            "Test".to_string(),
            past,
            "opencode".to_string(),
            "big-pickle".to_string(),
        ).unwrap();

        assert!(task.is_due());

        let future = Utc::now() + chrono::Duration::hours(1);
        let task2 = ScheduledTask::new_once(
            "Test".to_string(),
            future,
            "opencode".to_string(),
            "big-pickle".to_string(),
        ).unwrap();

        assert!(!task2.is_due());
    }

    #[test]
    fn test_compute_next_run() {
        let task = ScheduledTask::new_cron(
            "Test".to_string(),
            "0 0 * * * *".to_string(),
            "opencode".to_string(),
            "big-pickle".to_string(),
        ).unwrap();

        let after = Utc::now();
        let next = task.compute_next_run(after);
        assert!(next.is_some());
        assert!(next.unwrap() > after);
    }
}

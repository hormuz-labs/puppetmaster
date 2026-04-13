-- Settings table (key-value store for bot settings)
CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Scheduled tasks table
CREATE TABLE IF NOT EXISTS scheduled_tasks (
    id TEXT PRIMARY KEY NOT NULL,
    kind TEXT NOT NULL CHECK (kind IN ('once', 'cron')),
    prompt TEXT NOT NULL,
    schedule_summary TEXT,
    run_at TEXT, -- ISO 8601 for one-time tasks
    cron_expr TEXT, -- cron expression for recurring tasks
    next_run_at DATETIME,
    last_run_at DATETIME,
    last_status TEXT DEFAULT 'idle' CHECK (last_status IN ('idle', 'running', 'success', 'error')),
    last_error TEXT,
    run_count INTEGER DEFAULT 0,
    model_provider TEXT NOT NULL,
    model_id TEXT NOT NULL,
    project_worktree TEXT,
    session_id TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Session cache for quick lookups
CREATE TABLE IF NOT EXISTS session_cache (
    id TEXT PRIMARY KEY NOT NULL,
    directory TEXT NOT NULL,
    worktree TEXT NOT NULL,
    title TEXT,
    last_updated INTEGER NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Index for session cache lookups
CREATE INDEX IF NOT EXISTS idx_session_cache_worktree ON session_cache(worktree);
CREATE INDEX IF NOT EXISTS idx_session_cache_updated ON session_cache(last_updated DESC);

-- Index for scheduled tasks
CREATE INDEX IF NOT EXISTS idx_scheduled_tasks_next_run ON scheduled_tasks(next_run_at);
CREATE INDEX IF NOT EXISTS idx_scheduled_tasks_status ON scheduled_tasks(last_status);

-- Pinned messages tracking
CREATE TABLE IF NOT EXISTS pinned_messages (
    chat_id INTEGER PRIMARY KEY NOT NULL,
    message_id INTEGER NOT NULL,
    project_worktree TEXT,
    session_id TEXT,
    context_used INTEGER DEFAULT 0,
    context_limit INTEGER DEFAULT 0,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Assistant run state for tracking active runs
CREATE TABLE IF NOT EXISTS assistant_runs (
    session_id TEXT PRIMARY KEY NOT NULL,
    started_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    agent TEXT,
    provider_id TEXT,
    model_id TEXT,
    has_completed_response BOOLEAN DEFAULT FALSE,
    status TEXT DEFAULT 'running' CHECK (status IN ('running', 'completed', 'error'))
);

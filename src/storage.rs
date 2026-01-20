use std::collections::HashMap;
use std::path::Path;

use anyhow::Result;
use chrono::Utc;
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};

use crate::types::{Attachment, SessionData, SessionMessage};

#[derive(Debug, sqlx::FromRow)]
struct SessionRow {
    id: String,
    title: String,
    agent_type: String,
    model: Option<String>,
    env_json: Option<String>,
    extra_args_json: Option<String>,
    hidden: i64,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, sqlx::FromRow)]
struct MessageRow {
    id: String,
    role: String,
    content: String,
    attachments_json: Option<String>,
    status: Option<String>,
    model: Option<String>,
    agent_type: Option<String>,
    timestamp: String,
}

#[derive(Debug, sqlx::FromRow)]
struct SessionIdRow {
    id: String,
}

#[derive(Clone)]
pub struct Db {
    pool: SqlitePool,
}

impl Db {
    pub async fn new(path: &str) -> Result<Self> {
        if let Some(parent) = Path::new(path).parent() {
            std::fs::create_dir_all(parent)?;
        }
        let url = format!("sqlite://{}", path);
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&url)
            .await?;
        let db = Self { pool };
        db.init().await?;
        Ok(db)
    }

    async fn init(&self) -> Result<()> {
        sqlx::query("PRAGMA foreign_keys = ON;")
            .execute(&self.pool)
            .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL,
                title TEXT NOT NULL,
                agent_type TEXT NOT NULL,
                model TEXT,
                env_json TEXT,
                extra_args_json TEXT,
                hidden INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS messages (
                id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL,
                user_id TEXT NOT NULL,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                attachments_json TEXT,
                status TEXT,
                model TEXT,
                agent_type TEXT,
                timestamp TEXT NOT NULL,
                FOREIGN KEY(session_id) REFERENCES sessions(id) ON DELETE CASCADE
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_messages_session ON messages(session_id);")
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn upsert_session(
        &self,
        user_id: &str,
        session_id: &str,
        title: Option<String>,
        agent_type: Option<String>,
        model: Option<String>,
        env: Option<HashMap<String, String>>,
        extra_args: Option<Vec<String>>,
        hidden: Option<bool>,
    ) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        let row = sqlx::query_as::<_, SessionRow>(
            r#"SELECT id, title, agent_type, model, env_json, extra_args_json, hidden, created_at, updated_at
               FROM sessions WHERE id = ? AND user_id = ?"#,
        )
        .bind(session_id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        let (next_title, created_at) = match &row {
            Some(r) => {
                let next_title = title.unwrap_or(r.title.clone());
                (next_title, r.created_at.clone())
            }
            None => {
                let next_title = title.unwrap_or_else(|| "New Agent".to_string());
                (next_title, now.clone())
            }
        };

        let next_agent_type = agent_type
            .or_else(|| row.as_ref().map(|r| r.agent_type.clone()))
            .unwrap_or_else(|| "mock".to_string());
        let next_model = model.or_else(|| row.as_ref().and_then(|r| r.model.clone()));
        let next_env = env.or_else(|| row.as_ref().and_then(|r| parse_env(&r.env_json)));
        let next_args = extra_args.or_else(|| row.as_ref().and_then(|r| parse_args(&r.extra_args_json)));
        let next_hidden = hidden.unwrap_or_else(|| row.as_ref().map(|r| r.hidden != 0).unwrap_or(false));

        let env_json = serde_json::to_string(&next_env.unwrap_or_default())?;
        let extra_args_json = serde_json::to_string(&next_args.unwrap_or_default())?;

        sqlx::query(
            r#"
            INSERT INTO sessions (id, user_id, title, agent_type, model, env_json, extra_args_json, hidden, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET
                title = excluded.title,
                agent_type = excluded.agent_type,
                model = excluded.model,
                env_json = excluded.env_json,
                extra_args_json = excluded.extra_args_json,
                hidden = excluded.hidden,
                updated_at = excluded.updated_at
            "#,
        )
        .bind(session_id)
        .bind(user_id)
        .bind(next_title)
        .bind(next_agent_type)
        .bind(next_model)
        .bind(env_json)
        .bind(extra_args_json)
        .bind(if next_hidden { 1 } else { 0 })
        .bind(created_at)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn insert_message(
        &self,
        user_id: &str,
        session_id: &str,
        message: &SessionMessage,
    ) -> Result<()> {
        let attachments_json = serde_json::to_string(&message.attachments)?;
        sqlx::query(
            r#"
            INSERT INTO messages (id, session_id, user_id, role, content, attachments_json, status, model, agent_type, timestamp)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET
                content = excluded.content,
                attachments_json = excluded.attachments_json,
                status = excluded.status,
                model = excluded.model,
                agent_type = excluded.agent_type,
                timestamp = excluded.timestamp
            "#,
        )
        .bind(&message.id)
        .bind(session_id)
        .bind(user_id)
        .bind(&message.role)
        .bind(&message.content)
        .bind(attachments_json)
        .bind(&message.status)
        .bind(&message.model)
        .bind(&message.agent_type)
        .bind(&message.timestamp)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn list_sessions(&self, user_id: &str) -> Result<Vec<SessionData>> {
        let sessions = sqlx::query_as::<_, SessionRow>(
            r#"
            SELECT id, title, agent_type, model, env_json, extra_args_json, hidden, created_at, updated_at
            FROM sessions
            WHERE user_id = ?
            ORDER BY updated_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        let mut result = Vec::new();
        for s in sessions {
            let messages = self.list_messages(user_id, &s.id).await?;
            result.push(SessionData {
                id: s.id,
                title: s.title,
                agent_type: s.agent_type,
                model: s.model,
                env: parse_env(&s.env_json).unwrap_or_default(),
                extra_args: parse_args(&s.extra_args_json).unwrap_or_default(),
                hidden: s.hidden != 0,
                created_at: s.created_at,
                updated_at: s.updated_at,
                messages,
            });
        }
        Ok(result)
    }

    pub async fn list_session_ids(&self, user_id: &str) -> Result<Vec<String>> {
        let rows = sqlx::query_as::<_, SessionIdRow>(
            r#"
            SELECT id
            FROM sessions
            WHERE user_id = ?
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.id).collect())
    }

    pub async fn delete_session(&self, user_id: &str, session_id: &str) -> Result<()> {
        sqlx::query("DELETE FROM sessions WHERE id = ? AND user_id = ?")
        .bind(session_id)
        .bind(user_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn list_messages(&self, user_id: &str, session_id: &str) -> Result<Vec<SessionMessage>> {
        let rows = sqlx::query_as::<_, MessageRow>(
            r#"
            SELECT id, role, content, attachments_json, status, model, agent_type, timestamp
            FROM messages
            WHERE user_id = ? AND session_id = ?
            ORDER BY timestamp ASC
            "#,
        )
        .bind(user_id)
        .bind(session_id)
        .fetch_all(&self.pool)
        .await?;

        let mut messages = Vec::new();
        for row in rows {
            let attachments = parse_attachments(&row.attachments_json).unwrap_or_default();
            messages.push(SessionMessage {
                id: row.id,
                role: row.role,
                content: row.content,
                attachments,
                status: row.status,
                model: row.model,
                agent_type: row.agent_type,
                timestamp: row.timestamp,
            });
        }
        Ok(messages)
    }
}

fn parse_env(value: &Option<String>) -> Option<HashMap<String, String>> {
    value
        .as_ref()
        .and_then(|v| serde_json::from_str(v).ok())
}

fn parse_args(value: &Option<String>) -> Option<Vec<String>> {
    value
        .as_ref()
        .and_then(|v| serde_json::from_str(v).ok())
}

fn parse_attachments(value: &Option<String>) -> Option<Vec<Attachment>> {
    value
        .as_ref()
        .and_then(|v| serde_json::from_str(v).ok())
}

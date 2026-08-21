use chrono::{DateTime, Utc};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct ForumTopicRow {
    pub id: i64,
    pub title: String,
    pub author: String,
    pub posts: i64,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct ForumTopicHeadRow {
    pub id: i64,
    pub title: String,
    pub author: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct ForumPostRow {
    pub id: i64,
    pub content: String,
    pub author: String,
    pub created_at: DateTime<Utc>,
}

use chrono::{DateTime, Utc};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct WikiPageSummaryRow {
    pub slug: String,
    pub title: String,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct WikiPageRow {
    pub slug: String,
    pub title: String,
    pub content: String,
    pub author: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct WikiRevisionRow {
    pub id: i64,
    pub title: String,
    pub author: Option<String>,
    pub created_at: DateTime<Utc>,
}
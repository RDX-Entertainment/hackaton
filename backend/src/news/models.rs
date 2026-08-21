use chrono::{DateTime, Utc};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct NewsRow {
    pub id: i64,
    pub author: String,
    pub title: String,
    pub content: String,
    pub published_at: DateTime<Utc>,
    pub news_type: i16,
}

use chrono::{DateTime, Utc};
use serde::Serialize;

use super::models::NewsRow;

//
// ==============================
// NEWS FEED
// ==============================
//

#[derive(Debug, Serialize)]
pub struct NewsResponse {
    pub id: i64,
    pub author: String,
    pub title: String,
    pub content: String,
    pub published_at: DateTime<Utc>,

    // El legado lo llamaba `type`, que en Rust es palabra reservada; la columna sigue
    // llamandose asi y el JSON tambien, solo cambia el nombre del campo.
    #[serde(rename = "type")]
    pub news_type: i16,
}

impl From<NewsRow> for NewsResponse {
    fn from(value: NewsRow) -> Self {
        Self {
            id: value.id,
            author: value.author,
            title: value.title,
            content: value.content,
            published_at: value.published_at,
            news_type: value.news_type,
        }
    }
}

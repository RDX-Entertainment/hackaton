use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::models::{
    WikiPageRow,
    WikiPageSummaryRow,
    WikiRevisionRow,
};

//
// ==============================
// WIKI INDEX
// ==============================
//

#[derive(Debug, Serialize)]
pub struct WikiPageSummaryResponse {
    pub slug: String,
    pub title: String,
    pub updated_at: DateTime<Utc>,
}

impl From<WikiPageSummaryRow> for WikiPageSummaryResponse {
    fn from(value: WikiPageSummaryRow) -> Self {
        Self {
            slug: value.slug,
            title: value.title,
            updated_at: value.updated_at,
        }
    }
}

//
// ==============================
// WIKI PAGE
// ==============================
//

#[derive(Debug, Serialize)]
pub struct WikiPageResponse {
    pub slug: String,
    pub title: String,
    pub content: String,
    pub author: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<WikiPageRow> for WikiPageResponse {
    fn from(value: WikiPageRow) -> Self {
        Self {
            slug: value.slug,
            title: value.title,
            content: value.content,
            author: value.author,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

//
// ==============================
// EDIT
// ==============================
//

#[derive(Debug, Deserialize)]
pub struct WikiEditRequest {
    pub title: String,
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct WikiSavedResponse {
    pub slug: String,
    pub id: i64,
}

//
// ==============================
// REVISIONS
// ==============================
//

#[derive(Debug, Serialize)]
pub struct WikiRevisionResponse {
    pub id: i64,
    pub title: String,
    pub author: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl From<WikiRevisionRow> for WikiRevisionResponse {
    fn from(value: WikiRevisionRow) -> Self {
        Self {
            id: value.id,
            title: value.title,
            author: value.author,
            created_at: value.created_at,
        }
    }
}
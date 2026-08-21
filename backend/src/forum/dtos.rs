use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::models::{
    ForumPostRow,
    ForumTopicHeadRow,
    ForumTopicRow,
};

//
// ==============================
// TOPIC LIST
// ==============================
//

#[derive(Debug, Serialize)]
pub struct ForumTopicResponse {
    pub id: i64,
    pub title: String,
    pub author: String,
    pub posts: i64,
    pub updated_at: DateTime<Utc>,
}

impl From<ForumTopicRow> for ForumTopicResponse {
    fn from(value: ForumTopicRow) -> Self {
        Self {
            id: value.id,
            title: value.title,
            author: value.author,
            posts: value.posts,
            updated_at: value.updated_at,
        }
    }
}

//
// ==============================
// POSTS
// ==============================
//

#[derive(Debug, Serialize)]
pub struct ForumPostResponse {
    pub id: i64,
    pub content: String,
    pub author: String,
    pub created_at: DateTime<Utc>,
}

impl From<ForumPostRow> for ForumPostResponse {
    fn from(value: ForumPostRow) -> Self {
        Self {
            id: value.id,
            content: value.content,
            author: value.author,
            created_at: value.created_at,
        }
    }
}

//
// ==============================
// THREAD
// ==============================
//

#[derive(Debug, Serialize)]
pub struct ForumThreadResponse {
    pub id: i64,
    pub title: String,
    pub author: String,
    pub created_at: DateTime<Utc>,
    pub posts: Vec<ForumPostResponse>,
}

impl ForumThreadResponse {
    pub fn new(
        head: ForumTopicHeadRow,
        posts: Vec<ForumPostRow>,
    ) -> Self {
        Self {
            id: head.id,
            title: head.title,
            author: head.author,
            created_at: head.created_at,
            posts: posts
                .into_iter()
                .map(ForumPostResponse::from)
                .collect(),
        }
    }
}

//
// ==============================
// WRITING
// ==============================
//

#[derive(Debug, Deserialize)]
pub struct NewTopicRequest {
    pub title: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct NewPostRequest {
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct CreatedResponse {
    pub id: i64,
}

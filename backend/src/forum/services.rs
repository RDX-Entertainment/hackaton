use sqlx::PgPool;

use crate::error::ApiError;

use super::{
    dtos::{
        CreatedResponse,
        ForumThreadResponse,
        ForumTopicResponse,
        NewPostRequest,
        NewTopicRequest,
    },
    repo,
};

//
// ==============================
// TOPIC LIST
// ==============================
//

pub async fn topics(
    pool: &PgPool,
) -> Result<Vec<ForumTopicResponse>, ApiError> {
    let topics = repo::find_topics(pool)
        .await?
        .into_iter()
        .map(ForumTopicResponse::from)
        .collect();

    Ok(topics)
}

//
// ==============================
// THREAD
// ==============================
//

pub async fn thread(
    pool: &PgPool,
    topic_id: i64,
) -> Result<ForumThreadResponse, ApiError> {
    let head =
        repo::find_topic_head(pool, topic_id)
            .await?
            .ok_or(ApiError::NotFound)?;

    let posts =
        repo::find_posts(pool, topic_id).await?;

    Ok(ForumThreadResponse::new(head, posts))
}

//
// ==============================
// WRITING
// ==============================
//

const TITLE_MAX: usize = 120;

/// Un tema nuevo nace con su primer mensaje: los dos van en la misma transaccion para no
/// dejar temas vacios si algo falla.
pub async fn create_topic(
    pool: &PgPool,
    user_id: i64,
    input: NewTopicRequest,
) -> Result<CreatedResponse, ApiError> {
    let title = input.title.trim();
    let content = input.content.trim();

    if title.is_empty()
        || title.len() > TITLE_MAX
        || content.is_empty()
    {
        return Err(ApiError::BadRequest(
            "invalid forum topic".into(),
        ));
    }

    let mut tx = pool.begin().await?;

    let id = repo::insert_topic(
        &mut tx,
        title,
        user_id,
    )
    .await?;

    repo::insert_post(
        &mut tx,
        id,
        user_id,
        content,
    )
    .await?;

    tx.commit().await?;

    Ok(CreatedResponse { id })
}

pub async fn create_post(
    pool: &PgPool,
    user_id: i64,
    topic_id: i64,
    input: NewPostRequest,
) -> Result<CreatedResponse, ApiError> {
    let content = input.content.trim();

    if content.is_empty() {
        return Err(ApiError::BadRequest(
            "empty forum post".into(),
        ));
    }

    let mut tx = pool.begin().await?;

    if !repo::topic_exists(&mut tx, topic_id)
        .await?
    {
        return Err(ApiError::NotFound);
    }

    let id = repo::insert_post(
        &mut tx,
        topic_id,
        user_id,
        content,
    )
    .await?;

    repo::touch_topic(&mut tx, topic_id).await?;

    tx.commit().await?;

    Ok(CreatedResponse { id })
}

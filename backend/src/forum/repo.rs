use sqlx::{PgPool, Postgres, Transaction};

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

pub async fn find_topics(
    pool: &PgPool,
) -> Result<Vec<ForumTopicRow>, sqlx::Error> {
    sqlx::query_as::<_, ForumTopicRow>(
        r#"
        SELECT
            t.id,
            t.title,
            t.updated_at,
            u.login AS author,
            COUNT(p.id) AS posts
        FROM forum_topics t

        JOIN users u
            ON u.id = t.author_id

        LEFT JOIN forum_posts p
            ON p.topic_id = t.id

        GROUP BY t.id, u.login
        ORDER BY t.updated_at DESC
        "#,
    )
    .fetch_all(pool)
    .await
}

//
// ==============================
// THREAD
// ==============================
//

pub async fn find_topic_head(
    pool: &PgPool,
    topic_id: i64,
) -> Result<Option<ForumTopicHeadRow>, sqlx::Error>
{
    sqlx::query_as::<_, ForumTopicHeadRow>(
        r#"
        SELECT
            t.id,
            t.title,
            t.created_at,
            u.login AS author
        FROM forum_topics t

        JOIN users u
            ON u.id = t.author_id

        WHERE t.id = $1
        "#,
    )
    .bind(topic_id)
    .fetch_optional(pool)
    .await
}

pub async fn find_posts(
    pool: &PgPool,
    topic_id: i64,
) -> Result<Vec<ForumPostRow>, sqlx::Error> {
    sqlx::query_as::<_, ForumPostRow>(
        r#"
        SELECT
            p.id,
            p.content,
            p.created_at,
            u.login AS author
        FROM forum_posts p

        JOIN users u
            ON u.id = p.author_id

        WHERE p.topic_id = $1

        ORDER BY p.created_at
        "#,
    )
    .bind(topic_id)
    .fetch_all(pool)
    .await
}

//
// ==============================
// WRITING
// ==============================
//

pub async fn insert_topic(
    tx: &mut Transaction<'_, Postgres>,
    title: &str,
    author_id: i64,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        INSERT INTO forum_topics(
            title,
            author_id
        )
        VALUES($1, $2)
        RETURNING id
        "#,
    )
    .bind(title)
    .bind(author_id)
    .fetch_one(&mut **tx)
    .await
}

pub async fn insert_post(
    tx: &mut Transaction<'_, Postgres>,
    topic_id: i64,
    author_id: i64,
    content: &str,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        INSERT INTO forum_posts(
            topic_id,
            author_id,
            content
        )
        VALUES($1, $2, $3)
        RETURNING id
        "#,
    )
    .bind(topic_id)
    .bind(author_id)
    .bind(content)
    .fetch_one(&mut **tx)
    .await
}

pub async fn topic_exists(
    tx: &mut Transaction<'_, Postgres>,
    topic_id: i64,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        SELECT EXISTS(
            SELECT 1
            FROM forum_topics
            WHERE id = $1
        )
        "#,
    )
    .bind(topic_id)
    .fetch_one(&mut **tx)
    .await
}

/// Responder sube el tema en la lista, que se ordena por esta marca.
pub async fn touch_topic(
    tx: &mut Transaction<'_, Postgres>,
    topic_id: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE forum_topics
        SET updated_at = now()
        WHERE id = $1
        "#,
    )
    .bind(topic_id)
    .execute(&mut **tx)
    .await?;

    Ok(())
}

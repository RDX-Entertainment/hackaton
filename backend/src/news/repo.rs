use sqlx::PgPool;

use super::models::NewsRow;

//
// ==============================
// NEWS FEED
// ==============================
//

pub async fn find_latest(
    pool: &PgPool,
    limit: i64,
) -> Result<Vec<NewsRow>, sqlx::Error> {
    sqlx::query_as::<_, NewsRow>(
        r#"
        SELECT
            id,
            author,
            title,
            content,
            published_at,
            type AS news_type
        FROM news
        ORDER BY published_at DESC
        LIMIT $1
        "#,
    )
    .bind(limit)
    .fetch_all(pool)
    .await
}

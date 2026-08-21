use serde_json::Value;
use sqlx::PgPool;

pub async fn server_stats(
    pool: &PgPool,
) -> Result<Option<Value>, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        SELECT payload
        FROM system_stats
        WHERE key = 'server'
        "#,
    )
    .fetch_optional(pool)
    .await
}
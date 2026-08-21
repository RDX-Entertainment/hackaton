use sqlx::{
    PgPool,
    Postgres,
    Transaction,
};

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

pub async fn find_all_pages(
    pool: &PgPool,
) -> Result<Vec<WikiPageSummaryRow>, sqlx::Error> {
    sqlx::query_as::<_, WikiPageSummaryRow>(
        r#"
        SELECT
            slug,
            title,
            updated_at
        FROM wiki_pages
        ORDER BY title
        "#,
    )
    .fetch_all(pool)
    .await
}

//
// ==============================
// WIKI PAGE
// ==============================
//

pub async fn find_page_by_slug(
    pool: &PgPool,
    slug: &str,
) -> Result<Option<WikiPageRow>, sqlx::Error> {
    sqlx::query_as::<_, WikiPageRow>(
        r#"
        SELECT
            p.slug,
            p.title,
            p.content,
            p.created_at,
            p.updated_at,
            u.login AS author
        FROM wiki_pages p
        LEFT JOIN users u
            ON u.id = p.author_id
        WHERE p.slug = $1
        "#,
    )
    .bind(slug)
    .fetch_optional(pool)
    .await
}

//
// ==============================
// SAVE PAGE
// ==============================
//

pub async fn upsert_page(
    tx: &mut Transaction<'_, Postgres>,
    slug: &str,
    title: &str,
    content: &str,
    author_id: i64,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        INSERT INTO wiki_pages(
            slug,
            title,
            content,
            author_id
        )
        VALUES($1, $2, $3, $4)

        ON CONFLICT(slug)
        DO UPDATE SET
            title = EXCLUDED.title,
            content = EXCLUDED.content,
            author_id = EXCLUDED.author_id,
            updated_at = now()

        RETURNING id
        "#,
    )
    .bind(slug)
    .bind(title)
    .bind(content)
    .bind(author_id)
    .fetch_one(&mut **tx)
    .await
}

pub async fn insert_revision(
    tx: &mut Transaction<'_, Postgres>,
    page_id: i64,
    author_id: i64,
    title: &str,
    content: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO wiki_revisions(
            page_id,
            author_id,
            title,
            content
        )
        VALUES($1, $2, $3, $4)
        "#,
    )
    .bind(page_id)
    .bind(author_id)
    .bind(title)
    .bind(content)
    .execute(&mut **tx)
    .await?;

    Ok(())
}

//
// ==============================
// REVISIONS
// ==============================
//

pub async fn find_revisions_by_slug(
    pool: &PgPool,
    slug: &str,
) -> Result<Vec<WikiRevisionRow>, sqlx::Error> {
    sqlx::query_as::<_, WikiRevisionRow>(
        r#"
        SELECT
            r.id,
            r.title,
            r.created_at,
            u.login AS author
        FROM wiki_revisions r

        JOIN wiki_pages p
            ON p.id = r.page_id

        LEFT JOIN users u
            ON u.id = r.author_id

        WHERE p.slug = $1

        ORDER BY r.created_at DESC
        "#,
    )
    .bind(slug)
    .fetch_all(pool)
    .await
}
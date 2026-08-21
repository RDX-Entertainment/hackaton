use sqlx::PgPool;

use crate::error::ApiError;

use super::{
    dtos::{
        WikiEditRequest,
        WikiPageResponse,
        WikiPageSummaryResponse,
        WikiRevisionResponse,
        WikiSavedResponse,
    },
    repo,
};

//
// ==============================
// WIKI INDEX
// ==============================
//

pub async fn wiki_index(
    pool: &PgPool,
) -> Result<Vec<WikiPageSummaryResponse>, ApiError> {
    let pages = repo::find_all_pages(pool)
        .await?
        .into_iter()
        .map(WikiPageSummaryResponse::from)
        .collect();

    Ok(pages)
}

//
// ==============================
// WIKI PAGE
// ==============================
//

pub async fn wiki_page(
    pool: &PgPool,
    slug: &str,
) -> Result<WikiPageResponse, ApiError> {
    let page = repo::find_page_by_slug(
        pool,
        slug,
    )
    .await?
    .ok_or(ApiError::NotFound)?;

    Ok(page.into())
}

//
// ==============================
// SAVE PAGE
// ==============================
//

pub async fn save_wiki_page(
    pool: &PgPool,
    user_id: i64,
    slug: String,
    input: WikiEditRequest,
) -> Result<WikiSavedResponse, ApiError> {
    let title = input.title.trim();
    let content = input.content.trim();

    if !valid_slug(&slug)
        || title.is_empty()
        || title.len() > 160
        || content.is_empty()
    {
        return Err(
            ApiError::BadRequest(
                "invalid wiki page".into(),
            ),
        );
    }

    let mut tx = pool.begin().await?;

    let id = repo::upsert_page(
        &mut tx,
        &slug,
        title,
        content,
        user_id,
    )
    .await?;

    repo::insert_revision(
        &mut tx,
        id,
        user_id,
        title,
        content,
    )
    .await?;

    tx.commit().await?;

    Ok(WikiSavedResponse {
        slug,
        id,
    })
}

//
// ==============================
// REVISIONS
// ==============================
//

pub async fn wiki_revisions(
    pool: &PgPool,
    slug: &str,
) -> Result<Vec<WikiRevisionResponse>, ApiError> {
    let revisions =
        repo::find_revisions_by_slug(
            pool,
            slug,
        )
        .await?
        .into_iter()
        .map(WikiRevisionResponse::from)
        .collect();

    Ok(revisions)
}

//
// ==============================
// VALIDATION
// ==============================
//

fn valid_slug(slug: &str) -> bool {
    if slug.is_empty() || slug.len() > 160 {
        return false;
    }

    slug.chars().all(|c| {
        c.is_ascii_lowercase()
            || c.is_ascii_digit()
            || c == '-'
            || c == '_'
    })
}
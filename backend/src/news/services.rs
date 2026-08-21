use sqlx::PgPool;

use crate::error::ApiError;

use super::{
    dtos::NewsResponse,
    repo,
};

//
// ==============================
// NEWS FEED
// ==============================
//

/// Cuantas entradas devuelve el feed. El legado tiraba de las ultimas cincuenta.
const FEED_SIZE: i64 = 50;

pub async fn news_feed(
    pool: &PgPool,
) -> Result<Vec<NewsResponse>, ApiError> {
    let news = repo::find_latest(
        pool,
        FEED_SIZE,
    )
    .await?
    .into_iter()
    .map(NewsResponse::from)
    .collect();

    Ok(news)
}

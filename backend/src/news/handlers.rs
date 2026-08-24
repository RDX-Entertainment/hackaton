use actix_web::{
    web,
    HttpResponse,
};

use crate::{
    auth::Ctx,
    error::ApiError,
    state::AppState,
};

use super::services;

//
// ==============================
// ROUTES
// ==============================
//

pub fn configure(
    cfg: &mut web::ServiceConfig,
) {
    cfg.route(
        "/news",
        web::get().to(news),
    );
}

//
// ==============================
// FEED
// ==============================
//

pub async fn news(
    state: web::Data<AppState>,
    _: Ctx,
) -> Result<HttpResponse, ApiError> {
    let response =
        services::news_feed(
            &state.pool,
        )
        .await?;

    Ok(
        HttpResponse::Ok()
            .json(response),
    )
}

use actix_web::{
    web,
    HttpResponse,
};

use crate::{
    auth::CurrentUser,
    error::ApiError,
    state::AppState,
};

use super::{
    dtos::WikiEditRequest,
    services,
};

//
// ==============================
// ROUTES
// ==============================
//

pub fn configure(
    cfg: &mut web::ServiceConfig,
) {
    cfg
        .route(
            "/wiki",
            web::get().to(wiki_index),
        )
        .route(
            "/wiki/{slug}",
            web::get().to(wiki_page),
        )
        .route(
            "/wiki/{slug}",
            web::put().to(save_wiki_page),
        )
        .route(
            "/wiki/{slug}/revisions",
            web::get().to(wiki_revisions),
        );
}

//
// ==============================
// INDEX
// ==============================
//

pub async fn wiki_index(
    state: web::Data<AppState>,
    _: CurrentUser,
) -> Result<HttpResponse, ApiError> {
    let response =
        services::wiki_index(
            &state.pool,
        )
        .await?;

    Ok(
        HttpResponse::Ok()
            .json(response),
    )
}

//
// ==============================
// PAGE
// ==============================
//

pub async fn wiki_page(
    state: web::Data<AppState>,
    _: CurrentUser,
    slug: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let slug = slug.into_inner();

    let response =
        services::wiki_page(
            &state.pool,
            &slug,
        )
        .await?;

    Ok(
        HttpResponse::Ok()
            .json(response),
    )
}

//
// ==============================
// SAVE
// ==============================
//

pub async fn save_wiki_page(
    state: web::Data<AppState>,
    user: CurrentUser,
    slug: web::Path<String>,
    input: web::Json<WikiEditRequest>,
) -> Result<HttpResponse, ApiError> {
    let response =
        services::save_wiki_page(
            &state.pool,
            user.0,
            slug.into_inner(),
            input.into_inner(),
        )
        .await?;

    Ok(
        HttpResponse::Ok()
            .json(response),
    )
}

//
// ==============================
// REVISIONS
// ==============================
//

pub async fn wiki_revisions(
    state: web::Data<AppState>,
    _: CurrentUser,
    slug: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let slug = slug.into_inner();

    let response =
        services::wiki_revisions(
            &state.pool,
            &slug,
        )
        .await?;

    Ok(
        HttpResponse::Ok()
            .json(response),
    )
}
use actix_web::{
    web,
    HttpResponse,
};

use crate::{
    auth::Ctx,
    error::ApiError,
    state::AppState,
};

use super::{
    dtos::{NewPostRequest, NewTopicRequest},
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
            "/forum/topics",
            web::get().to(forum_topics),
        )
        .route(
            "/forum/topics",
            web::post().to(create_forum_topic),
        )
        .route(
            "/forum/topics/{id}",
            web::get().to(forum_topic),
        )
        .route(
            "/forum/topics/{id}/posts",
            web::post().to(create_forum_post),
        );
}

//
// ==============================
// TOPIC LIST
// ==============================
//

pub async fn forum_topics(
    state: web::Data<AppState>,
    _: Ctx,
) -> Result<HttpResponse, ApiError> {
    let response =
        services::topics(&state.pool).await?;

    Ok(
        HttpResponse::Ok()
            .json(response),
    )
}

//
// ==============================
// THREAD
// ==============================
//

pub async fn forum_topic(
    state: web::Data<AppState>,
    _: Ctx,
    id: web::Path<i64>,
) -> Result<HttpResponse, ApiError> {
    let response = services::thread(
        &state.pool,
        id.into_inner(),
    )
    .await?;

    Ok(
        HttpResponse::Ok()
            .json(response),
    )
}

//
// ==============================
// WRITING
// ==============================
//

pub async fn create_forum_topic(
    state: web::Data<AppState>,
    ctx: Ctx,
    input: web::Json<NewTopicRequest>,
) -> Result<HttpResponse, ApiError> {
    let response = services::create_topic(
        &state.pool,
        ctx.id,
        input.into_inner(),
    )
    .await?;

    Ok(
        HttpResponse::Created()
            .json(response),
    )
}

pub async fn create_forum_post(
    state: web::Data<AppState>,
    ctx: Ctx,
    id: web::Path<i64>,
    input: web::Json<NewPostRequest>,
) -> Result<HttpResponse, ApiError> {
    let response = services::create_post(
        &state.pool,
        ctx.id,
        id.into_inner(),
        input.into_inner(),
    )
    .await?;

    Ok(
        HttpResponse::Created()
            .json(response),
    )
}

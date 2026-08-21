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
    _: CurrentUser,
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
    _: CurrentUser,
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
    user: CurrentUser,
    input: web::Json<NewTopicRequest>,
) -> Result<HttpResponse, ApiError> {
    let response = services::create_topic(
        &state.pool,
        user.0,
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
    user: CurrentUser,
    id: web::Path<i64>,
    input: web::Json<NewPostRequest>,
) -> Result<HttpResponse, ApiError> {
    let response = services::create_post(
        &state.pool,
        user.0,
        id.into_inner(),
        input.into_inner(),
    )
    .await?;

    Ok(
        HttpResponse::Created()
            .json(response),
    )
}

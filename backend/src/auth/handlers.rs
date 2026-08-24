use actix_web::{
    web,
    HttpResponse,
};

use crate::{
    error::ApiError,
    state::AppState,
};

use super::{
    dtos::{LoginRequest, RegisterRequest},
    middleware::Ctx,
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
            "/auth/login",
            web::post().to(login),
        )
        .route(
            "/auth/register",
            web::post().to(register),
        )
        .route(
            "/me",
            web::get().to(me),
        );
}

//
// ==============================
// LOGIN
// ==============================
//

pub async fn login(
    state: web::Data<AppState>,
    input: web::Json<LoginRequest>,
) -> Result<HttpResponse, ApiError> {
    let response = services::login(
        &state.pool,
        &state.jwt_secret,
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
// REGISTER
// ==============================
//

pub async fn register(
    state: web::Data<AppState>,
    input: web::Json<RegisterRequest>,
) -> Result<HttpResponse, ApiError> {
    let response = services::register(
        &state.pool,
        &state.jwt_secret,
        input.into_inner(),
    )
    .await?;

    Ok(
        HttpResponse::Created()
            .json(response),
    )
}

//
// ==============================
// CURRENT USER
// ==============================
//

pub async fn me(
    state: web::Data<AppState>,
    ctx: Ctx,
) -> Result<HttpResponse, ApiError> {
    let response = services::me(
        &state.pool,
        ctx.id,
    )
    .await?;

    Ok(
        HttpResponse::Ok()
            .json(response),
    )
}

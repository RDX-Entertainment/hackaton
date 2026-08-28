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
    dtos::StartProcessRequest,
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
            "/game/process-actions",
            web::get().to(process_actions),
        )
        .route(
            "/processes",
            web::get().to(processes),
        )
        .route(
            "/processes",
            web::post().to(start_process),
        )
        .route(
            "/processes/{id}",
            web::get().to(process),
        )
        .route(
            "/processes/{id}",
            web::delete().to(delete_process),
        )
        .route(
            "/processes/{id}/pause",
            web::post().to(pause_process),
        )
        .route(
            "/processes/{id}/resume",
            web::post().to(resume_process),
        );
}

pub async fn process(
    state: web::Data<AppState>,
    ctx: Ctx,
    id: web::Path<i64>,
) -> Result<HttpResponse, ApiError> {
    let response = services::process(
        &state.pool,
        ctx.id,
        id.into_inner(),
    )
    .await?;

    Ok(HttpResponse::Ok().json(response))
}

//
// ==============================
// ACTION CATALOGUE
// ==============================
//

pub async fn process_actions(
    _: Ctx,
) -> HttpResponse {
    HttpResponse::Ok()
        .json(services::action_catalog())
}

//
// ==============================
// QUEUE
// ==============================
//

pub async fn processes(
    state: web::Data<AppState>,
    ctx: Ctx,
) -> Result<HttpResponse, ApiError> {
    let response = services::processes(
        &state.pool,
        ctx.id,
    )
    .await?;

    Ok(
        HttpResponse::Ok()
            .json(response),
    )
}

//
// ==============================
// START
// ==============================
//

pub async fn start_process(
    state: web::Data<AppState>,
    ctx: Ctx,
    input: web::Json<StartProcessRequest>,
) -> Result<HttpResponse, ApiError> {
    let response = services::start(
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

//
// ==============================
// DELETE
// ==============================
//

pub async fn delete_process(
    state: web::Data<AppState>,
    ctx: Ctx,
    id: web::Path<i64>,
) -> Result<HttpResponse, ApiError> {
    services::delete(
        &state.pool,
        ctx.id,
        id.into_inner(),
    )
    .await?;

    Ok(
        HttpResponse::NoContent()
            .finish(),
    )
}

//
// ==============================
// PAUSE / RESUME
// ==============================
//

pub async fn pause_process(
    state: web::Data<AppState>,
    ctx: Ctx,
    id: web::Path<i64>,
) -> Result<HttpResponse, ApiError> {
    let response = services::pause(
        &state.pool,
        ctx.id,
        id.into_inner(),
    )
    .await?;

    Ok(
        HttpResponse::Ok()
            .json(response),
    )
}

pub async fn resume_process(
    state: web::Data<AppState>,
    ctx: Ctx,
    id: web::Path<i64>,
) -> Result<HttpResponse, ApiError> {
    let response = services::resume(
        &state.pool,
        ctx.id,
        id.into_inner(),
    )
    .await?;

    Ok(
        HttpResponse::Ok()
            .json(response),
    )
}

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
    dtos::MissionCommandRequest,
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
            "/missions",
            web::get().to(missions),
        )
        .route(
            "/missions/{id}",
            web::post().to(mission_command),
        );
}

//
// ==============================
// BOARD
// ==============================
//

pub async fn missions(
    state: web::Data<AppState>,
    ctx: Ctx,
) -> Result<HttpResponse, ApiError> {
    let response = services::missions(
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
// COMMANDS
// ==============================
//

pub async fn mission_command(
    state: web::Data<AppState>,
    ctx: Ctx,
    id: web::Path<i64>,
    input: web::Json<MissionCommandRequest>,
) -> Result<HttpResponse, ApiError> {
    let response = services::command(
        &state.pool,
        ctx.id,
        id.into_inner(),
        input.into_inner().action,
    )
    .await?;

    Ok(
        HttpResponse::Ok()
            .json(response),
    )
}

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
    user: CurrentUser,
) -> Result<HttpResponse, ApiError> {
    let response = services::missions(
        &state.pool,
        user.0,
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
    user: CurrentUser,
    id: web::Path<i64>,
    input: web::Json<MissionCommandRequest>,
) -> Result<HttpResponse, ApiError> {
    let response = services::command(
        &state.pool,
        user.0,
        id.into_inner(),
        input.into_inner().action,
    )
    .await?;

    Ok(
        HttpResponse::Ok()
            .json(response),
    )
}

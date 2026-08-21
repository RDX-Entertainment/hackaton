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
    dtos::BuyPartRequest,
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
            "/hardware",
            web::get().to(hardware),
        )
        .route(
            "/hardware/shop",
            web::get().to(hardware_shop),
        )
        .route(
            "/hardware/shop",
            web::post().to(buy_hardware),
        )
        .route(
            "/stats",
            web::get().to(server_stats),
        );
}

//
// ==============================
// MACHINES
// ==============================
//

pub async fn hardware(
    state: web::Data<AppState>,
    user: CurrentUser,
) -> Result<HttpResponse, ApiError> {
    let response = services::machines(
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
// SHOP
// ==============================
//

pub async fn hardware_shop(
    _: CurrentUser,
) -> HttpResponse {
    HttpResponse::Ok()
        .json(services::shop())
}

pub async fn buy_hardware(
    state: web::Data<AppState>,
    user: CurrentUser,
    input: web::Json<BuyPartRequest>,
) -> Result<HttpResponse, ApiError> {
    let response = services::buy(
        &state.pool,
        user.0,
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
// SERVER STATS
// ==============================
//

pub async fn server_stats(
    state: web::Data<AppState>,
    _: CurrentUser,
) -> Result<HttpResponse, ApiError> {
    let response =
        services::server_stats(&state.pool)
            .await?;

    Ok(
        HttpResponse::Ok()
            .json(response),
    )
}

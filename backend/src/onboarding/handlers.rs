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
        "/onboarding",
        web::get().to(onboarding),
    );
}

//
// ==============================
// ONBOARDING
// ==============================
//

/// En que paso esta el jugador, deducido del mundo. Solo lee.
pub async fn onboarding(
    state: web::Data<AppState>,
    ctx: Ctx,
) -> Result<HttpResponse, ApiError> {
    let response = services::onboarding(
        &state.pool,
        ctx.id,
    )
    .await?;

    Ok(
        HttpResponse::Ok()
            .json(response),
    )
}

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
    dtos::{
        TutorialConfigInput,
        TutorialStepInput,
    },
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

//
// ==============================
// EL PANEL
// ==============================
//

/// Se monta dentro de `/api/admin`, protegido por `AdminOnly` en main.
pub fn configure_admin(
    cfg: &mut web::ServiceConfig,
) {
    cfg.route(
        "/tutorial",
        web::get().to(tutorial),
    )
    .route(
        "/tutorial/vocabulario",
        web::get().to(vocabulary),
    )
    .route(
        "/tutorial/config",
        web::put().to(set_config),
    )
    .route(
        "/tutorial/steps",
        web::post().to(create_step),
    )
    .route(
        "/tutorial/steps/{id}",
        web::put().to(edit_step),
    )
    .route(
        "/tutorial/steps/{id}",
        web::delete().to(remove_step),
    );
}

/// El tutorial entero: el interruptor y los pasos, encendidos y apagados.
pub async fn tutorial(
    state: web::Data<AppState>,
) -> Result<HttpResponse, ApiError> {
    let response = services::tutorial(&state.pool).await?;

    Ok(HttpResponse::Ok().json(response))
}

/// Lo que se puede elegir, dicho por quien lo va a comprobar.
pub async fn vocabulary() -> Result<HttpResponse, ApiError> {
    Ok(
        HttpResponse::Ok()
            .json(services::vocabulary()),
    )
}

pub async fn set_config(
    state: web::Data<AppState>,
    input: web::Json<TutorialConfigInput>,
) -> Result<HttpResponse, ApiError> {
    services::set_config(
        &state.pool,
        input.into_inner(),
    )
    .await?;

    Ok(HttpResponse::NoContent().finish())
}

pub async fn create_step(
    state: web::Data<AppState>,
    input: web::Json<TutorialStepInput>,
) -> Result<HttpResponse, ApiError> {
    let id = services::save_step(
        &state.pool,
        None,
        input.into_inner(),
    )
    .await?;

    Ok(
        HttpResponse::Created()
            .json(serde_json::json!({ "id": id })),
    )
}

pub async fn edit_step(
    state: web::Data<AppState>,
    id: web::Path<i64>,
    input: web::Json<TutorialStepInput>,
) -> Result<HttpResponse, ApiError> {
    services::save_step(
        &state.pool,
        Some(id.into_inner()),
        input.into_inner(),
    )
    .await?;

    Ok(HttpResponse::NoContent().finish())
}

pub async fn remove_step(
    state: web::Data<AppState>,
    id: web::Path<i64>,
) -> Result<HttpResponse, ApiError> {
    services::delete_step(
        &state.pool,
        id.into_inner(),
    )
    .await?;

    Ok(HttpResponse::NoContent().finish())
}

use actix_web::{web, HttpResponse};
use serde::Deserialize;

use crate::{auth::Ctx, error::ApiError, state::AppState};

use super::{
    dtos::{BuyKitRequest, KitInput},
    services,
};

//
// ==============================
// ROUTES
// ==============================
//

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/hardware/kits", web::get().to(kits))
        .route("/hardware/kits/{id}/buy", web::post().to(buy_kit));
}

/// Rutas del panel. Se montan dentro del scope `/admin`, que main envuelve en `AdminOnly`:
/// ningun handler de aqui comprueba permisos por su cuenta porque para quien no es
/// administrador estas rutas no existen.
pub fn configure_admin(cfg: &mut web::ServiceConfig) {
    cfg.route("/kits", web::get().to(admin_kits))
        .route("/kits", web::post().to(admin_create_kit))
        .route("/kits/{id}", web::put().to(admin_edit_kit))
        .route("/kits/{id}", web::delete().to(admin_delete_kit))
        .route("/kits/{id}/default", web::put().to(admin_set_default))
        .route("/clans", web::get().to(admin_clans));
}

//
// ==============================
// JUGADOR
// ==============================
//

pub async fn kits(
    state: web::Data<AppState>,
    ctx: Ctx,
) -> Result<HttpResponse, ApiError> {
    let response = services::offered(&state.pool, ctx.id).await?;

    Ok(HttpResponse::Ok().json(response))
}

pub async fn buy_kit(
    state: web::Data<AppState>,
    ctx: Ctx,
    id: web::Path<i64>,
    input: web::Json<BuyKitRequest>,
) -> Result<HttpResponse, ApiError> {
    let response = services::buy(
        &state.pool,
        ctx.id,
        id.into_inner(),
        input.into_inner(),
    )
    .await?;

    Ok(HttpResponse::Ok().json(response))
}

//
// ==============================
// PANEL
// ==============================
//

pub async fn admin_kits(
    state: web::Data<AppState>,
    ctx: Ctx,
) -> Result<HttpResponse, ApiError> {
    let response = services::list(&state.pool, ctx).await?;

    Ok(HttpResponse::Ok().json(response))
}

pub async fn admin_create_kit(
    state: web::Data<AppState>,
    ctx: Ctx,
    input: web::Json<KitInput>,
) -> Result<HttpResponse, ApiError> {
    let id = services::create(&state.pool, ctx, input.into_inner()).await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({ "id": id })))
}

pub async fn admin_edit_kit(
    state: web::Data<AppState>,
    ctx: Ctx,
    id: web::Path<i64>,
    input: web::Json<KitInput>,
) -> Result<HttpResponse, ApiError> {
    services::edit(&state.pool, ctx, id.into_inner(), input.into_inner()).await?;

    Ok(HttpResponse::NoContent().finish())
}

pub async fn admin_delete_kit(
    state: web::Data<AppState>,
    ctx: Ctx,
    id: web::Path<i64>,
) -> Result<HttpResponse, ApiError> {
    services::remove(&state.pool, ctx, id.into_inner()).await?;

    Ok(HttpResponse::NoContent().finish())
}

#[derive(Debug, Deserialize)]
pub struct DefaultInput {
    pub on: bool,
}

pub async fn admin_set_default(
    state: web::Data<AppState>,
    ctx: Ctx,
    id: web::Path<i64>,
    input: web::Json<DefaultInput>,
) -> Result<HttpResponse, ApiError> {
    services::set_default(&state.pool, ctx, id.into_inner(), input.on).await?;

    Ok(HttpResponse::NoContent().finish())
}

pub async fn admin_clans(
    state: web::Data<AppState>,
    ctx: Ctx,
) -> Result<HttpResponse, ApiError> {
    let response = services::clans(&state.pool, ctx).await?;

    Ok(HttpResponse::Ok().json(response))
}

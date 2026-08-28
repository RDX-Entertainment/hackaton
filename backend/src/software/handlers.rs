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
        NewSoftwareRequest,
        AdminSoftwareRequest,
        AdminSoftwareRunningRequest,
        ResearchStartRequest,
        TransferRequest,
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
    cfg
        //
        // Software types
        //
        .route(
            "/game/software-types",
            web::get().to(software_type_catalog),
        )

        //
        // Local software
        //
        .route(
            "/software",
            web::get().to(software),
        )
        .route(
            "/software",
            web::post().to(create_software),
        )

        //
        // Research
        //
        .route(
            "/research",
            web::get().to(research),
        )
        .route(
            "/research",
            web::post().to(start_research),
        );

    //
    // Las rutas de:
    //
    // remote_software
    // transfer_software
    // delete_remote_software
    //
    // no estaban en el fragmento de rutas que me pasaste.
    //
    // NO las inventamos.
    //
}

//
// ==============================
// SOFTWARE TYPES
// ==============================
//

pub async fn software_type_catalog(
    _: Ctx,
) -> HttpResponse {
    HttpResponse::Ok().json(
        services::software_type_catalog(),
    )
}

//
// ==============================
// SOFTWARE
// ==============================
//

pub async fn software(
    state: web::Data<AppState>,
    ctx: Ctx,
) -> Result<HttpResponse, ApiError> {
    let response = services::software(
        &state.pool,
        ctx.id,
    )
    .await?;

    Ok(
        HttpResponse::Ok()
            .json(response),
    )
}

pub async fn create_software(
    state: web::Data<AppState>,
    ctx: Ctx,
    input: web::Json<NewSoftwareRequest>,
) -> Result<HttpResponse, ApiError> {
    let response = services::create_software(
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
// RESEARCH
// ==============================
//

pub async fn research(
    state: web::Data<AppState>,
    ctx: Ctx,
) -> Result<HttpResponse, ApiError> {
    let response = services::research(
        &state.pool,
        ctx.id,
    )
    .await?;

    Ok(
        HttpResponse::Ok()
            .json(response),
    )
}

pub async fn start_research(
    state: web::Data<AppState>,
    ctx: Ctx,
    input: web::Json<ResearchStartRequest>,
) -> Result<HttpResponse, ApiError> {
    let response = services::start_research(
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

/// Las rutas se montan dentro del scope /admin, protegido por AdminOnly en main.
pub fn configure_admin(cfg: &mut web::ServiceConfig) {
    cfg
        .route("/users/{user}/software", web::get().to(admin_software))
        .route("/users/{user}/software", web::post().to(admin_create_software))
        .route("/users/{user}/software/{software}", web::put().to(admin_update_software))
        .route("/users/{user}/software/{software}", web::delete().to(admin_delete_software))
        .route("/users/{user}/software/{software}/running", web::put().to(admin_set_software_running));
}

pub async fn admin_software(state: web::Data<AppState>, _: Ctx, user: web::Path<i64>) -> Result<HttpResponse, ApiError> {
    Ok(HttpResponse::Ok().json(services::admin_software(&state.pool, user.into_inner()).await?))
}

pub async fn admin_create_software(state: web::Data<AppState>, _: Ctx, user: web::Path<i64>, input: web::Json<AdminSoftwareRequest>) -> Result<HttpResponse, ApiError> {
    let id = services::admin_create_software(&state.pool, user.into_inner(), input.into_inner()).await?;
    Ok(HttpResponse::Created().json(serde_json::json!({ "id": id })))
}

pub async fn admin_update_software(state: web::Data<AppState>, _: Ctx, path: web::Path<(i64, i64)>, input: web::Json<AdminSoftwareRequest>) -> Result<HttpResponse, ApiError> {
    let (user, software) = path.into_inner();
    services::admin_update_software(&state.pool, user, software, input.into_inner()).await?;
    Ok(HttpResponse::NoContent().finish())
}

pub async fn admin_delete_software(state: web::Data<AppState>, _: Ctx, path: web::Path<(i64, i64)>) -> Result<HttpResponse, ApiError> {
    let (user, software) = path.into_inner();
    services::admin_delete_software(&state.pool, user, software).await?;
    Ok(HttpResponse::NoContent().finish())
}

pub async fn admin_set_software_running(state: web::Data<AppState>, _: Ctx, path: web::Path<(i64, i64)>, input: web::Json<AdminSoftwareRunningRequest>) -> Result<HttpResponse, ApiError> {
    let (user, software) = path.into_inner();
    services::admin_set_software_running(&state.pool, user, software, input.running).await?;
    Ok(HttpResponse::NoContent().finish())
}

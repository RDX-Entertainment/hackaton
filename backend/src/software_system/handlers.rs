use super::{
    dtos::{CompileRequest, CreateBlueprintRequest, StartOperationRequest, TuneProgramRequest},
    services,
};
use crate::{auth::Ctx, error::ApiError, state::AppState};
use actix_web::{web, HttpResponse};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/software-system/capabilities", web::get().to(capabilities))
        .route("/software-lab/programs", web::get().to(lab_programs))
        .route("/software-lab/programs/{id}", web::put().to(tune_program))
        .route("/software-system/research/modules", web::get().to(modules))
        .route("/software-system/programs", web::get().to(programs))
        .route("/software-system/blueprints", web::get().to(blueprints))
        .route(
            "/software-system/blueprints",
            web::post().to(create_blueprint),
        )
        .route("/software-system/compile", web::post().to(compile))
        .route(
            "/software-system/operations",
            web::post().to(start_operation),
        );
}
async fn capabilities(state: web::Data<AppState>, _: Ctx) -> Result<HttpResponse, ApiError> {
    Ok(HttpResponse::Ok().json(services::capabilities(&state.pool).await?))
}
async fn lab_programs(state: web::Data<AppState>, ctx: Ctx) -> Result<HttpResponse, ApiError> {
    Ok(HttpResponse::Ok().json(services::programs(&state.pool, ctx.id).await?))
}
async fn tune_program(
    state: web::Data<AppState>,
    ctx: Ctx,
    id: web::Path<i64>,
    input: web::Json<TuneProgramRequest>,
) -> Result<HttpResponse, ApiError> {
    services::tune_program(
        &state.pool,
        ctx.id,
        id.into_inner(),
        input.into_inner().stats,
    )
    .await?;
    Ok(HttpResponse::NoContent().finish())
}
async fn modules(state: web::Data<AppState>, ctx: Ctx) -> Result<HttpResponse, ApiError> {
    Ok(HttpResponse::Ok().json(services::modules(&state.pool, ctx.id).await?))
}
async fn programs(state: web::Data<AppState>, ctx: Ctx) -> Result<HttpResponse, ApiError> {
    Ok(HttpResponse::Ok().json(services::programs(&state.pool, ctx.id).await?))
}
async fn blueprints(state: web::Data<AppState>, ctx: Ctx) -> Result<HttpResponse, ApiError> {
    Ok(HttpResponse::Ok().json(services::blueprints(&state.pool, ctx.id).await?))
}
async fn create_blueprint(
    state: web::Data<AppState>,
    ctx: Ctx,
    input: web::Json<CreateBlueprintRequest>,
) -> Result<HttpResponse, ApiError> {
    Ok(HttpResponse::Created()
        .json(services::create_blueprint(&state.pool, ctx.id, input.into_inner()).await?))
}
async fn compile(
    state: web::Data<AppState>,
    ctx: Ctx,
    input: web::Json<CompileRequest>,
) -> Result<HttpResponse, ApiError> {
    Ok(HttpResponse::Created()
        .json(services::compile(&state.pool, ctx.id, input.blueprint_id).await?))
}
async fn start_operation(
    state: web::Data<AppState>,
    ctx: Ctx,
    input: web::Json<StartOperationRequest>,
) -> Result<HttpResponse, ApiError> {
    Ok(HttpResponse::Created()
        .json(services::start_operation(&state.pool, ctx.id, input.into_inner()).await?))
}

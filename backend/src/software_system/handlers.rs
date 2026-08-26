use super::{
    blueprints,
    catalog,
    dtos::{CompileRequest, CreateBlueprintRequest, StartOperationRequest},
    operations,
};
use crate::{auth::Ctx, error::ApiError, state::AppState};
use actix_web::{HttpResponse, web};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/software-system/capabilities", web::get().to(capabilities))
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
    Ok(HttpResponse::Ok().json(catalog::capabilities(&state.pool).await?))
}
async fn modules(state: web::Data<AppState>, ctx: Ctx) -> Result<HttpResponse, ApiError> {
    Ok(HttpResponse::Ok().json(catalog::modules(&state.pool, ctx.id).await?))
}
async fn programs(state: web::Data<AppState>, ctx: Ctx) -> Result<HttpResponse, ApiError> {
    Ok(HttpResponse::Ok().json(catalog::programs(&state.pool, ctx.id).await?))
}
async fn blueprints(state: web::Data<AppState>, ctx: Ctx) -> Result<HttpResponse, ApiError> {
    Ok(HttpResponse::Ok().json(blueprints::list(&state.pool, ctx.id).await?))
}
async fn create_blueprint(
    state: web::Data<AppState>,
    ctx: Ctx,
    input: web::Json<CreateBlueprintRequest>,
) -> Result<HttpResponse, ApiError> {
    Ok(HttpResponse::Created()
        .json(blueprints::create(&state.pool, ctx.id, input.into_inner()).await?))
}
async fn compile(
    state: web::Data<AppState>,
    ctx: Ctx,
    input: web::Json<CompileRequest>,
) -> Result<HttpResponse, ApiError> {
    Ok(HttpResponse::Created()
        .json(blueprints::compile(&state.pool, ctx.id, input.blueprint_id).await?))
}
async fn start_operation(
    state: web::Data<AppState>,
    ctx: Ctx,
    input: web::Json<StartOperationRequest>,
) -> Result<HttpResponse, ApiError> {
    Ok(HttpResponse::Created()
        .json(operations::start(&state.pool, ctx.id, input.into_inner()).await?))
}

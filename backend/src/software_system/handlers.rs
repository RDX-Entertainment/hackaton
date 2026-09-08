use super::{
    dtos::{
        CompileRequest, CreateBlueprintRequest, ForgeEvidenceRequest, ModifyWebRequest,
        StartOperationRequest, TuneProgramRequest,
    },
    services,
};
use crate::{auth::Ctx, error::ApiError, state::AppState};
use actix_web::{HttpResponse, web};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/software-system/capabilities", web::get().to(capabilities))
        .route("/software-system/accesses", web::get().to(accesses))
        .route(
            "/software-system/accesses/{ip}/web",
            web::get().to(web_content),
        )
        .route(
            "/software-system/accesses/{ip}/web",
            web::put().to(modify_web),
        )
        .route(
            "/software-system/accesses/{ip}/evidence/{id}/hide",
            web::post().to(hide_evidence),
        )
        .route(
            "/software-system/accesses/{ip}/evidence/{id}/forge",
            web::post().to(forge_evidence),
        )
        .route("/software-system/evidence", web::get().to(evidence))
        .route(
            "/software-system/evidence/discover",
            web::post().to(discover_evidence),
        )
        .route(
            "/software-system/evidence/{id}/analyze",
            web::post().to(analyze_evidence),
        )
        .route("/software-lab/programs", web::get().to(lab_programs))
        .route("/software-lab/programs/{id}", web::put().to(tune_program))
        .route("/software-system/research/modules", web::get().to(modules))
        .route("/software-system/programs", web::get().to(programs))
        .route("/software-system/blueprints", web::get().to(blueprints))
        .route("/software-system/blueprints/preview", web::post().to(preview_blueprint))
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
async fn modify_web(
    state: web::Data<AppState>,
    ctx: Ctx,
    ip: web::Path<i64>,
    input: web::Json<ModifyWebRequest>,
) -> Result<HttpResponse, ApiError> {
    Ok(HttpResponse::Created().json(
        services::modify_web(&state.pool, ctx.id, ip.into_inner(), input.into_inner()).await?,
    ))
}
async fn hide_evidence(
    state: web::Data<AppState>,
    ctx: Ctx,
    path: web::Path<(i64, i64)>,
) -> Result<HttpResponse, ApiError> {
    let (ip, evidence) = path.into_inner();
    Ok(HttpResponse::Created()
        .json(services::hide_evidence(&state.pool, ctx.id, ip, evidence).await?))
}
async fn forge_evidence(
    state: web::Data<AppState>,
    ctx: Ctx,
    path: web::Path<(i64, i64)>,
    input: web::Json<ForgeEvidenceRequest>,
) -> Result<HttpResponse, ApiError> {
    let (ip, evidence) = path.into_inner();
    Ok(HttpResponse::Created().json(
        services::forge_evidence(&state.pool, ctx.id, ip, evidence, input.into_inner()).await?,
    ))
}
async fn accesses(state: web::Data<AppState>, ctx: Ctx) -> Result<HttpResponse, ApiError> {
    Ok(HttpResponse::Ok().json(services::accesses(&state.pool, ctx.id).await?))
}
async fn web_content(
    state: web::Data<AppState>,
    ctx: Ctx,
    ip: web::Path<i64>,
) -> Result<HttpResponse, ApiError> {
    Ok(HttpResponse::Ok().json(services::web_content(&state.pool, ctx.id, ip.into_inner()).await?))
}
async fn evidence(state: web::Data<AppState>, ctx: Ctx) -> Result<HttpResponse, ApiError> {
    Ok(HttpResponse::Ok().json(services::evidence(&state.pool, ctx.id).await?))
}
async fn analyze_evidence(
    state: web::Data<AppState>,
    ctx: Ctx,
    id: web::Path<i64>,
) -> Result<HttpResponse, ApiError> {
    Ok(HttpResponse::Created()
        .json(services::analyze_evidence(&state.pool, ctx.id, id.into_inner()).await?))
}
async fn discover_evidence(
    state: web::Data<AppState>,
    ctx: Ctx,
) -> Result<HttpResponse, ApiError> {
    Ok(HttpResponse::Created().json(services::discover_evidence(&state.pool, ctx.id).await?))
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
async fn preview_blueprint(
    state: web::Data<AppState>,
    ctx: Ctx,
    input: web::Json<CreateBlueprintRequest>,
) -> Result<HttpResponse, ApiError> {
    Ok(HttpResponse::Ok().json(services::preview_blueprint(&state.pool, ctx.id, &input).await?))
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

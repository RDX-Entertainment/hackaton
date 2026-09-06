use actix_web::{
    web,
    HttpResponse,
};

use crate::{
    auth::Ctx,
    error::ApiError,
    software::{
        dtos::TransferRequest,
        services as software_services,
    },
    state::AppState,
};

use super::{
    dtos::{
        InternetHackRequest,
        InternetLoginRequest,
        IpRequest,
        LogEditRequest,
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
        .route(
            "/internet/targets",
            web::get().to(internet_targets),
        )
        .route(
            "/internet/history",
            web::get().to(internet_history),
        )
        .route(
            "/internet/bookmarks",
            web::get().to(internet_bookmarks),
        )
        .route(
            "/internet/bookmarks",
            web::post().to(add_bookmark),
        )
        .route(
            "/internet/bookmarks/{ip}",
            web::delete().to(remove_bookmark),
        )
        .route(
            "/internet/login",
            web::post().to(internet_login),
        )
        .route(
            "/internet/hack",
            web::post().to(internet_hack),
        )
        .route(
            "/internet/analyze",
            web::post().to(internet_analyze),
        )

        //
        // Remote software
        //
        .route(
            "/internet/{ip}/software",
            web::get().to(remote_software),
        )
        .route(
            "/internet/{ip}/software",
            web::post().to(transfer_software),
        )
        .route(
            "/internet/{ip}/software/{soft}",
            web::delete().to(delete_remote_software),
        )

        //
        // Remote log
        //
        .route(
            "/internet/{ip}/log",
            web::get().to(remote_log),
        )
        .route(
            "/internet/{ip}/log",
            web::put().to(edit_remote_log),
        )

        //
        // Navigation
        //
        .route(
            "/internet/{ip}",
            web::get().to(internet_navigate),
        );
}

//
// ==============================
// TARGETS
// ==============================
//

pub async fn internet_targets(
    state: web::Data<AppState>,
    ctx: Ctx,
) -> Result<HttpResponse, ApiError> {
    let response =
        services::internet_targets(
            &state.pool,
            ctx.id,
        )
        .await?;

    Ok(HttpResponse::Ok().json(response))
}

//
// ==============================
// NAVIGATION
// ==============================
//

pub async fn internet_navigate(
    state: web::Data<AppState>,
    ctx: Ctx,
    ip: web::Path<i64>,
) -> Result<HttpResponse, ApiError> {
    let response =
        services::internet_navigate(
            &state.pool,
            ctx.id,
            ip.into_inner(),
        )
        .await?;

    Ok(HttpResponse::Ok().json(response))
}

//
// ==============================
// LOGIN
// ==============================
//

pub async fn internet_login(
    state: web::Data<AppState>,
    ctx: Ctx,
    input: web::Json<InternetLoginRequest>,
) -> Result<HttpResponse, ApiError> {
    let response =
        services::internet_login(
            &state.pool,
            ctx.id,
            input.into_inner(),
        )
        .await?;

    Ok(HttpResponse::Ok().json(response))
}

//
// ==============================
// HACK
// ==============================
//

pub async fn internet_hack(
    state: web::Data<AppState>,
    ctx: Ctx,
    input: web::Json<InternetHackRequest>,
) -> Result<HttpResponse, ApiError> {
    let response =
        services::internet_hack(
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
// ANALYZE
// ==============================
//

pub async fn internet_analyze(
    state: web::Data<AppState>,
    ctx: Ctx,
    input: web::Json<IpRequest>,
) -> Result<HttpResponse, ApiError> {
    let response =
        services::internet_analyze(
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
// REMOTE SOFTWARE
// ==============================
//

pub async fn remote_software(
    state: web::Data<AppState>,
    ctx: Ctx,
    ip: web::Path<i64>,
) -> Result<HttpResponse, ApiError> {
    let ip = ip.into_inner();

    let target =
        services::resolve_ip(
            &state.pool,
            ip,
            ctx.id,
        )
        .await?;

    services::require_access(
        &state.pool,
        ctx.id,
        ip,
    )
    .await?;

    let response =
        software_services::remote_software(
            &state.pool,
            target.id,
            target.is_npc,
        )
        .await?;

    Ok(HttpResponse::Ok().json(response))
}

pub async fn transfer_software(
    state: web::Data<AppState>,
    ctx: Ctx,
    ip: web::Path<i64>,
    input: web::Json<TransferRequest>,
) -> Result<HttpResponse, ApiError> {
    let ip = ip.into_inner();

    let target =
        services::resolve_ip(
            &state.pool,
            ip,
            ctx.id,
        )
        .await?;

    services::require_access(
        &state.pool,
        ctx.id,
        ip,
    )
    .await?;

    let response =
        software_services::transfer_software(
            &state.pool,
            ctx.id,
            target.id,
            target.is_npc,
            target.server_id,
            input.into_inner(),
        )
        .await?;

    Ok(
        HttpResponse::Created()
            .json(response),
    )
}

pub async fn delete_remote_software(
    state: web::Data<AppState>,
    ctx: Ctx,
    path: web::Path<(i64, i64)>,
) -> Result<HttpResponse, ApiError> {
    let (ip, software_id) =
        path.into_inner();

    let target =
        services::resolve_ip(
            &state.pool,
            ip,
            ctx.id,
        )
        .await?;

    services::require_access(
        &state.pool,
        ctx.id,
        ip,
    )
    .await?;

    let response =
        software_services::delete_remote_software(
            &state.pool,
            ctx.id,
            target.id,
            target.is_npc,
            target.server_id,
            software_id,
        )
        .await?;

    Ok(
        HttpResponse::Created()
            .json(response),
    )
}

//
// ==============================
// REMOTE LOG
// ==============================
//

pub async fn remote_log(
    state: web::Data<AppState>,
    ctx: Ctx,
    ip: web::Path<i64>,
) -> Result<HttpResponse, ApiError> {
    let response =
        services::remote_log(
            &state.pool,
            ctx.id,
            ip.into_inner(),
        )
        .await?;

    Ok(HttpResponse::Ok().json(response))
}

pub async fn edit_remote_log(
    state: web::Data<AppState>,
    ctx: Ctx,
    ip: web::Path<i64>,
    input: web::Json<LogEditRequest>,
) -> Result<HttpResponse, ApiError> {
    let response =
        services::edit_remote_log(
            &state.pool,
            ctx.id,
            ip.into_inner(),
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
// BOOKMARKS
// ==============================
//

pub async fn internet_bookmarks(
    state: web::Data<AppState>,
    ctx: Ctx,
) -> Result<HttpResponse, ApiError> {
    let response =
        services::internet_bookmarks(
            &state.pool,
            ctx.id,
        )
        .await?;

    Ok(HttpResponse::Ok().json(response))
}

pub async fn add_bookmark(
    state: web::Data<AppState>,
    ctx: Ctx,
    input: web::Json<IpRequest>,
) -> Result<HttpResponse, ApiError> {
    let response =
        services::add_bookmark(
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

pub async fn remove_bookmark(
    state: web::Data<AppState>,
    ctx: Ctx,
    ip: web::Path<i64>,
) -> Result<HttpResponse, ApiError> {
    services::remove_bookmark(
        &state.pool,
        ctx.id,
        ip.into_inner(),
    )
    .await?;

    Ok(HttpResponse::NoContent().finish())
}

//
// ==============================
// HISTORY
// ==============================
//

pub async fn internet_history(
    state: web::Data<AppState>,
    ctx: Ctx,
) -> Result<HttpResponse, ApiError> {
    let response =
        services::internet_history(
            &state.pool,
            ctx.id,
        )
        .await?;

    Ok(HttpResponse::Ok().json(response))
}
//
// ==============================
// SITIOS · PANEL
// ==============================
//

/// Se montan dentro del scope `/admin`, que main envuelve en `AdminOnly`.
pub fn configure_admin(cfg: &mut web::ServiceConfig) {
    cfg.route("/sites", web::get().to(admin_sites))
        .route("/sites/{kind}/{id}", web::put().to(admin_set_site))
        // Quien puede usar cada orden de un sitio. Es lo que hace que la banca integrada
        // sea una concesion y no una regla escrita en Rust.
        .route("/site-actions", web::get().to(site_actions))
        .route("/site-grants", web::get().to(list_grants))
        .route("/site-grants", web::post().to(create_grant))
        .route("/site-grants/{id}", web::delete().to(remove_grant));
}

pub async fn admin_sites(
    state: web::Data<AppState>,
    ctx: Ctx,
) -> Result<HttpResponse, ApiError> {
    let response = services::all_sites(&state.pool, ctx).await?;

    Ok(HttpResponse::Ok().json(response))
}

pub async fn admin_set_site(
    state: web::Data<AppState>,
    ctx: Ctx,
    path: web::Path<(String, i64)>,
    input: web::Json<super::dtos::SiteUpdate>,
) -> Result<HttpResponse, ApiError> {
    let (kind, id) = path.into_inner();

    services::set_site(&state.pool, ctx, &kind, id, &input.web).await?;

    Ok(HttpResponse::NoContent().finish())
}


//
// ==============================
// CONCESIONES DE ORDENES
// ==============================
//

/// El catalogo entero, para que el panel sepa que existe y de que familia es cada cosa.
pub async fn site_actions(ctx: Ctx) -> Result<HttpResponse, ApiError> {
    ctx.require_admin()?;

    Ok(HttpResponse::Ok().json(super::grants::catalog_actions()))
}

pub async fn list_grants(
    state: web::Data<AppState>,
    ctx: Ctx,
) -> Result<HttpResponse, ApiError> {
    Ok(HttpResponse::Ok().json(super::grants::list(&state.pool, ctx).await?))
}

pub async fn create_grant(
    state: web::Data<AppState>,
    ctx: Ctx,
    input: web::Json<super::grants::GrantInput>,
) -> Result<HttpResponse, ApiError> {
    let id = super::grants::create(&state.pool, ctx, input.into_inner()).await?;

    Ok(HttpResponse::Created().json(serde_json::json!({ "id": id })))
}

pub async fn remove_grant(
    state: web::Data<AppState>,
    ctx: Ctx,
    path: web::Path<i64>,
) -> Result<HttpResponse, ApiError> {
    super::grants::remove(&state.pool, ctx, path.into_inner()).await?;

    Ok(HttpResponse::NoContent().finish())
}

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
        BuyMachineRequest,
        BuyPartRequest,
        ChipsetInput,
        GrantRequest,
        InstallRequest,
        LinkRequest,
        MoveMachineRequest,
        PartInput,
        RenameMachineRequest,
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
        .route("/hardware/machines/{id}/gateway", web::put().to(set_gateway))
        .route("/hardware/switches", web::post().to(buy_switch))
        .route(
            "/hardware",
            web::get().to(hardware),
        )
        .route(
            "/hardware/machines",
            web::get().to(machine_detail),
        )
        .route(
            "/hardware/machines",
            web::post().to(buy_machine),
        )
        .route(
            "/hardware/machines/quote",
            web::get().to(machine_quote),
        )
        .route(
            "/hardware/machines/{id}",
            web::put().to(rename_machine),
        )
        .route(
            "/hardware/machines/{id}",
            web::delete().to(scrap_machine),
        )
        .route(
            "/hardware/components",
            web::get().to(components),
        )
        .route(
            "/hardware/components/{id}/install",
            web::post().to(install_component),
        )
        .route(
            "/hardware/components/{id}/install",
            web::delete().to(uninstall_component),
        )
        .route(
            "/hardware/shop",
            web::get().to(hardware_shop),
        )
        .route(
            "/hardware/shop",
            web::post().to(buy_hardware),
        )
        // ── El mapa de nodos ────────────────────────────────────────────────
        //
        // La posicion cuelga de la maquina y no de un recurso "mapa": es un
        // atributo suyo, como el nombre. Los cables si son recurso propio,
        // porque no pertenecen a ninguno de los dos extremos.
        .route(
            "/hardware/machines/{id}/position",
            web::put().to(move_machine),
        )
        .route(
            "/hardware/links",
            web::get().to(links),
        )
        .route(
            "/hardware/links",
            web::post().to(create_link),
        )
        .route(
            "/hardware/clusters",
            web::post().to(create_cluster),
        )
        .route(
            "/hardware/clusters/{id}",
            web::delete().to(dissolve_cluster),
        )
        .route(
            "/hardware/links/{id}",
            web::delete().to(delete_link),
        )
        .route(
            "/stats",
            web::get().to(server_stats),
        );
}

//
// ==============================
// MAPA DE NODOS
// ==============================
//

pub async fn move_machine(
    state: web::Data<AppState>,
    ctx: Ctx,
    id: web::Path<i64>,
    input: web::Json<MoveMachineRequest>,
) -> Result<HttpResponse, ApiError> {
    services::move_machine(
        &state.pool,
        ctx.id,
        id.into_inner(),
        input.into_inner(),
    )
    .await?;

    Ok(HttpResponse::NoContent().finish())
}

pub async fn buy_switch(state: web::Data<AppState>, ctx: Ctx, input: web::Json<BuyMachineRequest>) -> Result<HttpResponse, ApiError> {
    Ok(HttpResponse::Ok().json(services::buy_switch(&state.pool, ctx.id, input.into_inner()).await?))
}

pub async fn set_gateway(state: web::Data<AppState>, ctx: Ctx, id: web::Path<i64>) -> Result<HttpResponse, ApiError> {
    services::set_gateway(&state.pool, ctx.id, id.into_inner()).await?;
    Ok(HttpResponse::NoContent().finish())
}

pub async fn links(
    state: web::Data<AppState>,
    ctx: Ctx,
) -> Result<HttpResponse, ApiError> {
    let found = services::links(
        &state.pool,
        ctx.id,
    )
    .await?;

    Ok(HttpResponse::Ok().json(found))
}

pub async fn create_link(
    state: web::Data<AppState>,
    ctx: Ctx,
    input: web::Json<LinkRequest>,
) -> Result<HttpResponse, ApiError> {
    let created = services::create_link(
        &state.pool,
        ctx.id,
        input.into_inner(),
    )
    .await?;

    Ok(HttpResponse::Ok().json(created))
}

pub async fn delete_link(
    state: web::Data<AppState>,
    ctx: Ctx,
    id: web::Path<i64>,
) -> Result<HttpResponse, ApiError> {
    services::delete_link(
        &state.pool,
        ctx.id,
        id.into_inner(),
    )
    .await?;

    Ok(HttpResponse::NoContent().finish())
}

/// Rutas del panel para el catalogo de piezas. Se montan dentro del scope `/admin`, que
/// main envuelve en `AdminOnly`: ningun handler de aqui comprueba permisos por su cuenta
/// porque para quien no es administrador estas rutas no existen.
pub fn configure_admin(
    cfg: &mut web::ServiceConfig,
) {
    cfg
        .route(
            "/parts",
            web::get().to(admin_parts),
        )
        .route(
            "/parts",
            web::post().to(admin_create_part),
        )
        .route(
            "/parts/{id}",
            web::put().to(admin_edit_part),
        )
        .route(
            "/parts/{id}",
            web::delete().to(admin_delete_part),
        )
        .route(
            "/components",
            web::post().to(admin_grant),
        )
        .route(
            "/chipsets",
            web::get().to(admin_chipsets),
        )
        .route(
            "/chipsets",
            web::post().to(admin_create_chipset),
        )
        .route(
            "/chipsets/{id}",
            web::delete().to(admin_delete_chipset),
        );
}

//
// ==============================
// MACHINES
// ==============================
//

pub async fn hardware(
    state: web::Data<AppState>,
    ctx: Ctx,
) -> Result<HttpResponse, ApiError> {
    let response = services::machines(
        &state.pool,
        ctx.id,
    )
    .await?;

    Ok(
        HttpResponse::Ok()
            .json(response),
    )
}

/// Cada maquina con su placa, sus piezas y sus ranuras libres.
pub async fn machine_detail(
    state: web::Data<AppState>,
    ctx: Ctx,
) -> Result<HttpResponse, ApiError> {
    let response = services::machine_detail(
        &state.pool,
        ctx.id,
    )
    .await?;

    Ok(
        HttpResponse::Ok()
            .json(response),
    )
}

pub async fn machine_quote(
    state: web::Data<AppState>,
    ctx: Ctx,
) -> Result<HttpResponse, ApiError> {
    let response = services::machine_quote(
        &state.pool,
        ctx.id,
    )
    .await?;

    Ok(
        HttpResponse::Ok()
            .json(response),
    )
}

pub async fn buy_machine(
    state: web::Data<AppState>,
    ctx: Ctx,
    input: web::Json<BuyMachineRequest>,
) -> Result<HttpResponse, ApiError> {
    let response = services::buy_machine(
        &state.pool,
        ctx.id,
        input.into_inner(),
    )
    .await?;

    Ok(
        HttpResponse::Ok()
            .json(response),
    )
}

pub async fn rename_machine(
    state: web::Data<AppState>,
    ctx: Ctx,
    id: web::Path<i64>,
    input: web::Json<RenameMachineRequest>,
) -> Result<HttpResponse, ApiError> {
    services::rename_machine(
        &state.pool,
        ctx.id,
        id.into_inner(),
        input.into_inner(),
    )
    .await?;

    Ok(HttpResponse::NoContent().finish())
}

pub async fn scrap_machine(
    state: web::Data<AppState>,
    ctx: Ctx,
    id: web::Path<i64>,
) -> Result<HttpResponse, ApiError> {
    services::scrap_machine(
        &state.pool,
        ctx.id,
        id.into_inner(),
    )
    .await?;

    Ok(HttpResponse::NoContent().finish())
}

//
// ==============================
// COMPONENTS
// ==============================
//

pub async fn components(
    state: web::Data<AppState>,
    ctx: Ctx,
) -> Result<HttpResponse, ApiError> {
    let response = services::components(
        &state.pool,
        ctx.id,
    )
    .await?;

    Ok(
        HttpResponse::Ok()
            .json(response),
    )
}

pub async fn install_component(
    state: web::Data<AppState>,
    ctx: Ctx,
    id: web::Path<i64>,
    input: web::Json<InstallRequest>,
) -> Result<HttpResponse, ApiError> {
    let response = services::install(
        &state.pool,
        ctx.id,
        id.into_inner(),
        input.into_inner(),
    )
    .await?;

    Ok(
        HttpResponse::Ok()
            .json(response),
    )
}

pub async fn uninstall_component(
    state: web::Data<AppState>,
    ctx: Ctx,
    id: web::Path<i64>,
) -> Result<HttpResponse, ApiError> {
    let response = services::uninstall(
        &state.pool,
        ctx.id,
        id.into_inner(),
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
    state: web::Data<AppState>,
    _: Ctx,
) -> Result<HttpResponse, ApiError> {
    let response =
        services::shop(&state.pool).await?;

    Ok(
        HttpResponse::Ok()
            .json(response),
    )
}

pub async fn buy_hardware(
    state: web::Data<AppState>,
    ctx: Ctx,
    input: web::Json<BuyPartRequest>,
) -> Result<HttpResponse, ApiError> {
    let response = services::buy(
        &state.pool,
        ctx.id,
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
// ADMIN
// ==============================
//

pub async fn admin_parts(
    state: web::Data<AppState>,
    ctx: Ctx,
) -> Result<HttpResponse, ApiError> {
    let response = services::list_parts(
        &state.pool,
        ctx,
    )
    .await?;

    Ok(
        HttpResponse::Ok()
            .json(response),
    )
}

pub async fn admin_create_part(
    state: web::Data<AppState>,
    ctx: Ctx,
    input: web::Json<PartInput>,
) -> Result<HttpResponse, ApiError> {
    let id = services::create_part(
        &state.pool,
        ctx,
        input.into_inner(),
    )
    .await?;

    Ok(
        HttpResponse::Ok()
            .json(serde_json::json!({ "id": id })),
    )
}

pub async fn admin_edit_part(
    state: web::Data<AppState>,
    ctx: Ctx,
    id: web::Path<i64>,
    input: web::Json<PartInput>,
) -> Result<HttpResponse, ApiError> {
    services::edit_part(
        &state.pool,
        ctx,
        id.into_inner(),
        input.into_inner(),
    )
    .await?;

    Ok(HttpResponse::NoContent().finish())
}

pub async fn admin_delete_part(
    state: web::Data<AppState>,
    ctx: Ctx,
    id: web::Path<i64>,
) -> Result<HttpResponse, ApiError> {
    services::remove_part(
        &state.pool,
        ctx,
        id.into_inner(),
    )
    .await?;

    Ok(HttpResponse::NoContent().finish())
}

pub async fn admin_grant(
    state: web::Data<AppState>,
    ctx: Ctx,
    input: web::Json<GrantRequest>,
) -> Result<HttpResponse, ApiError> {
    let made = services::grant(
        &state.pool,
        ctx,
        input.into_inner(),
    )
    .await?;

    Ok(
        HttpResponse::Ok()
            .json(serde_json::json!({ "components": made })),
    )
}

//
// ==============================
// SERVER STATS
// ==============================
//

pub async fn server_stats(
    state: web::Data<AppState>,
    _: Ctx,
) -> Result<HttpResponse, ApiError> {
    let response =
        services::server_stats(&state.pool)
            .await?;

    Ok(
        HttpResponse::Ok()
            .json(response),
    )
}

pub async fn admin_chipsets(
    state: web::Data<AppState>,
    _: Ctx,
) -> Result<HttpResponse, ApiError> {
    let response =
        services::chipsets(&state.pool).await?;

    Ok(
        HttpResponse::Ok()
            .json(response),
    )
}

pub async fn admin_create_chipset(
    state: web::Data<AppState>,
    ctx: Ctx,
    input: web::Json<ChipsetInput>,
) -> Result<HttpResponse, ApiError> {
    let id = services::create_chipset(
        &state.pool,
        ctx,
        input.into_inner(),
    )
    .await?;

    Ok(
        HttpResponse::Ok()
            .json(serde_json::json!({ "id": id })),
    )
}

pub async fn admin_delete_chipset(
    state: web::Data<AppState>,
    ctx: Ctx,
    id: web::Path<i64>,
) -> Result<HttpResponse, ApiError> {
    services::remove_chipset(
        &state.pool,
        ctx,
        id.into_inner(),
    )
    .await?;

    Ok(HttpResponse::NoContent().finish())
}

/// Montar un grupo de equipos cableados.
pub async fn create_cluster(
    state: web::Data<AppState>,
    ctx: Ctx,
    input: web::Json<super::dtos::ClusterRequest>,
) -> Result<HttpResponse, ApiError> {
    let response = services::create_cluster(&state.pool, ctx.id, input.into_inner()).await?;

    Ok(HttpResponse::Created().json(response))
}

/// Deshacerlo. Solo lo hace el jugador; caerse el controlador no basta.
pub async fn dissolve_cluster(
    state: web::Data<AppState>,
    ctx: Ctx,
    path: web::Path<i64>,
) -> Result<HttpResponse, ApiError> {
    services::dissolve_cluster(&state.pool, ctx.id, path.into_inner()).await?;

    Ok(HttpResponse::NoContent().finish())
}

use actix_web::{HttpResponse, web};

use crate::{auth::Ctx, error::ApiError, state::AppState};

use super::{
    dtos::{ComputerBuildInput, CreateServerInput, NetworkInput, NetworkNpcInput, NodeInput, NodeLinkInput, OrganizationInput},
    services,
};

/// Se monta dentro de `/api/admin`, protegido por `AdminOnly` en main.
pub fn configure_admin(cfg: &mut web::ServiceConfig) {
    cfg.route("/organizations/npcs", web::get().to(npc_catalog))
        .route("/organizations", web::get().to(list))
        .route("/organizations", web::post().to(create))
        .route("/organizations/{id}/links", web::post().to(create_link))
        .route("/organizations/{id}/links/{link}", web::delete().to(remove_link))
        .route("/organizations/{id}", web::get().to(get))
        .route("/organizations/{id}", web::put().to(edit))
        .route("/organizations/{id}", web::delete().to(remove))
        .route(
            "/organizations/{id}/networks",
            web::post().to(create_network),
        )
        .route(
            "/organizations/{id}/networks/{network}",
            web::put().to(edit_network),
        )
        .route(
            "/organizations/{id}/networks/{network}",
            web::delete().to(remove_network),
        )
        .route(
            "/organizations/{id}/networks/{network}/npcs",
            web::post().to(assign_npc),
        )
        .route(
            "/organizations/{id}/networks/{network}/npcs/{npc}",
            web::put().to(edit_node),
        )
        .route(
            "/organizations/{id}/networks/{network}/npcs/{npc}/computer",
            web::get().to(get_computer),
        )
        .route(
            "/organizations/{id}/networks/{network}/npcs/{npc}/computer",
            web::put().to(save_computer),
        )
        .route(
            "/organizations/{id}/networks/{network}/npcs/{npc}/web",
            web::post().to(connect_web),
        )
        .route(
            "/organizations/{id}/networks/{network}/npcs/{npc}/web",
            web::delete().to(disconnect_web),
        )
        .route(
            "/organizations/{id}/networks/{network}/servers",
            web::post().to(create_server),
        );
}
async fn create_link(state: web::Data<AppState>, ctx: Ctx, id: web::Path<i64>, input: web::Json<NodeLinkInput>) -> Result<HttpResponse, ApiError> { let link = services::create_link(&state.pool, ctx, id.into_inner(), input.into_inner()).await?; Ok(HttpResponse::Created().json(serde_json::json!({"id":link}))) }
async fn remove_link(state: web::Data<AppState>, ctx: Ctx, path: web::Path<(i64,i64)>) -> Result<HttpResponse, ApiError> { let (organization,link)=path.into_inner(); services::remove_link(&state.pool,ctx,organization,link).await?; Ok(HttpResponse::NoContent().finish()) }

async fn edit_node(state: web::Data<AppState>, ctx: Ctx, path: web::Path<(i64, i64, i64)>, input: web::Json<NodeInput>) -> Result<HttpResponse, ApiError> {
    let (organization, network, npc) = path.into_inner();
    services::edit_node(&state.pool, ctx, organization, network, npc, input.into_inner()).await?;
    Ok(HttpResponse::NoContent().finish())
}

async fn save_computer(state: web::Data<AppState>, ctx: Ctx, path: web::Path<(i64, i64, i64)>, input: web::Json<ComputerBuildInput>) -> Result<HttpResponse, ApiError> {
    let (organization, network, npc) = path.into_inner();
    services::save_computer(&state.pool, ctx, organization, network, npc, input.into_inner()).await?;
    Ok(HttpResponse::NoContent().finish())
}

async fn get_computer(state: web::Data<AppState>, ctx: Ctx, path: web::Path<(i64, i64, i64)>) -> Result<HttpResponse, ApiError> {
    let (organization, network, npc) = path.into_inner();
    Ok(HttpResponse::Ok().json(services::computer_configuration(&state.pool, ctx, organization, network, npc).await?))
}

async fn connect_web(state: web::Data<AppState>, ctx: Ctx, path: web::Path<(i64, i64, i64)>) -> Result<HttpResponse, ApiError> {
    let (organization, network, npc) = path.into_inner();
    services::connect_web(&state.pool, ctx, organization, network, npc).await?;
    Ok(HttpResponse::NoContent().finish())
}

async fn disconnect_web(state: web::Data<AppState>, ctx: Ctx, path: web::Path<(i64, i64, i64)>) -> Result<HttpResponse, ApiError> {
    let (organization, network, npc) = path.into_inner();
    services::disconnect_web(&state.pool, ctx, organization, network, npc).await?;
    Ok(HttpResponse::NoContent().finish())
}

async fn npc_catalog(state: web::Data<AppState>, ctx: Ctx) -> Result<HttpResponse, ApiError> {
    Ok(HttpResponse::Ok().json(services::npc_catalog(&state.pool, ctx).await?))
}

async fn list(state: web::Data<AppState>, ctx: Ctx) -> Result<HttpResponse, ApiError> {
    Ok(HttpResponse::Ok().json(services::list(&state.pool, ctx).await?))
}

async fn get(
    state: web::Data<AppState>,
    ctx: Ctx,
    id: web::Path<i64>,
) -> Result<HttpResponse, ApiError> {
    Ok(HttpResponse::Ok().json(services::get(&state.pool, ctx, id.into_inner()).await?))
}

async fn create(
    state: web::Data<AppState>,
    ctx: Ctx,
    input: web::Json<OrganizationInput>,
) -> Result<HttpResponse, ApiError> {
    let id = services::create(&state.pool, ctx, input.into_inner()).await?;
    Ok(HttpResponse::Created().json(serde_json::json!({ "id": id })))
}

async fn edit(
    state: web::Data<AppState>,
    ctx: Ctx,
    id: web::Path<i64>,
    input: web::Json<OrganizationInput>,
) -> Result<HttpResponse, ApiError> {
    services::edit(&state.pool, ctx, id.into_inner(), input.into_inner()).await?;
    Ok(HttpResponse::NoContent().finish())
}

async fn remove(
    state: web::Data<AppState>,
    ctx: Ctx,
    id: web::Path<i64>,
) -> Result<HttpResponse, ApiError> {
    services::remove(&state.pool, ctx, id.into_inner()).await?;
    Ok(HttpResponse::NoContent().finish())
}

async fn create_network(
    state: web::Data<AppState>,
    ctx: Ctx,
    id: web::Path<i64>,
    input: web::Json<NetworkInput>,
) -> Result<HttpResponse, ApiError> {
    let network =
        services::create_network(&state.pool, ctx, id.into_inner(), input.into_inner()).await?;
    Ok(HttpResponse::Created().json(serde_json::json!({ "id": network })))
}

async fn edit_network(
    state: web::Data<AppState>,
    ctx: Ctx,
    path: web::Path<(i64, i64)>,
    input: web::Json<NetworkInput>,
) -> Result<HttpResponse, ApiError> {
    let (organization, network) = path.into_inner();
    services::edit_network(&state.pool, ctx, organization, network, input.into_inner()).await?;
    Ok(HttpResponse::NoContent().finish())
}

async fn remove_network(
    state: web::Data<AppState>,
    ctx: Ctx,
    path: web::Path<(i64, i64)>,
) -> Result<HttpResponse, ApiError> {
    let (organization, network) = path.into_inner();
    services::remove_network(&state.pool, ctx, organization, network).await?;
    Ok(HttpResponse::NoContent().finish())
}

async fn assign_npc(
    state: web::Data<AppState>,
    ctx: Ctx,
    path: web::Path<(i64, i64)>,
    input: web::Json<NetworkNpcInput>,
) -> Result<HttpResponse, ApiError> {
    let (organization, network) = path.into_inner();
    services::assign_network_npc(&state.pool, ctx, organization, network, input.into_inner())
        .await?;
    Ok(HttpResponse::NoContent().finish())
}

async fn create_server(
    state: web::Data<AppState>,
    ctx: Ctx,
    path: web::Path<(i64, i64)>,
    input: web::Json<CreateServerInput>,
) -> Result<HttpResponse, ApiError> {
    let (organization, network) = path.into_inner();
    let server =
        services::create_server(&state.pool, ctx, organization, network, input.into_inner())
            .await?;
    Ok(HttpResponse::Created().json(server))
}

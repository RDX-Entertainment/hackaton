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
        ClanJoinRequest,
        CreateClanRequest,
        FriendRequestRequest,
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
            "/ranking/users",
            web::get().to(ranking),
        )
        .route(
            "/friends",
            web::get().to(friends),
        )
        .route(
            "/friends/requests",
            web::post().to(request_friend),
        )
        .route(
            "/friends/requests/{id}/accept",
            web::post().to(accept_friend),
        )
        .route(
            "/clans",
            web::get().to(clans),
        )
        .route(
            "/clans",
            web::post().to(create_clan),
        )
        .route(
            "/clans/mine",
            web::get().to(my_clan),
        )
        .route(
            "/clans/requests",
            web::get().to(clan_requests),
        )
        .route(
            "/clans/{id}/join",
            web::post().to(request_clan_join),
        )
        .route(
            "/clans/requests/{id}/accept",
            web::post().to(accept_clan_request),
        );
}

//
// ==============================
// RANKING
// ==============================
//

pub async fn ranking(
    state: web::Data<AppState>,
    _: Ctx,
) -> Result<HttpResponse, ApiError> {
    let response =
        services::ranking(&state.pool).await?;

    Ok(
        HttpResponse::Ok()
            .json(response),
    )
}

//
// ==============================
// FRIENDS
// ==============================
//

pub async fn friends(
    state: web::Data<AppState>,
    ctx: Ctx,
) -> Result<HttpResponse, ApiError> {
    let response = services::friends(
        &state.pool,
        ctx.id,
    )
    .await?;

    Ok(
        HttpResponse::Ok()
            .json(response),
    )
}

pub async fn request_friend(
    state: web::Data<AppState>,
    ctx: Ctx,
    input: web::Json<FriendRequestRequest>,
) -> Result<HttpResponse, ApiError> {
    let response = services::request_friend(
        &state.pool,
        ctx.id,
        input.into_inner().user_id,
    )
    .await?;

    Ok(
        HttpResponse::Created()
            .json(response),
    )
}

pub async fn accept_friend(
    state: web::Data<AppState>,
    ctx: Ctx,
    id: web::Path<i64>,
) -> Result<HttpResponse, ApiError> {
    let response = services::accept_friend(
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
// CLAN LIST
// ==============================
//

pub async fn clans(
    state: web::Data<AppState>,
    _: Ctx,
) -> Result<HttpResponse, ApiError> {
    let response =
        services::clans(&state.pool).await?;

    Ok(
        HttpResponse::Ok()
            .json(response),
    )
}

pub async fn my_clan(
    state: web::Data<AppState>,
    ctx: Ctx,
) -> Result<HttpResponse, ApiError> {
    let response = services::my_clan(
        &state.pool,
        ctx.id,
    )
    .await?;

    // Sin clan la respuesta es null, que es lo que la pantalla espera para ofrecer crear
    // uno en vez de pintar el propio.
    Ok(
        HttpResponse::Ok()
            .json(response),
    )
}

//
// ==============================
// CLAN WRITING
// ==============================
//

pub async fn create_clan(
    state: web::Data<AppState>,
    ctx: Ctx,
    input: web::Json<CreateClanRequest>,
) -> Result<HttpResponse, ApiError> {
    let response = services::create_clan(
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
// JOIN REQUESTS
// ==============================
//

pub async fn request_clan_join(
    state: web::Data<AppState>,
    ctx: Ctx,
    id: web::Path<i64>,
    input: web::Json<ClanJoinRequest>,
) -> Result<HttpResponse, ApiError> {
    let response = services::request_join(
        &state.pool,
        ctx.id,
        id.into_inner(),
        input.into_inner(),
    )
    .await?;

    Ok(
        HttpResponse::Created()
            .json(response),
    )
}

pub async fn clan_requests(
    state: web::Data<AppState>,
    ctx: Ctx,
) -> Result<HttpResponse, ApiError> {
    let response = services::requests(
        &state.pool,
        ctx.id,
    )
    .await?;

    Ok(
        HttpResponse::Ok()
            .json(response),
    )
}

pub async fn accept_clan_request(
    state: web::Data<AppState>,
    ctx: Ctx,
    id: web::Path<i64>,
) -> Result<HttpResponse, ApiError> {
    let response = services::accept_request(
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

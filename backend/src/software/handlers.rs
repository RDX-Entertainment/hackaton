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
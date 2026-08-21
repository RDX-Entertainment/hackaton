use actix_web::{
    web,
    HttpResponse,
};

use crate::{
    auth::CurrentUser,
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
    _: CurrentUser,
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
    user: CurrentUser,
) -> Result<HttpResponse, ApiError> {
    let response = services::software(
        &state.pool,
        user.0,
    )
    .await?;

    Ok(
        HttpResponse::Ok()
            .json(response),
    )
}

pub async fn create_software(
    state: web::Data<AppState>,
    user: CurrentUser,
    input: web::Json<NewSoftwareRequest>,
) -> Result<HttpResponse, ApiError> {
    let response = services::create_software(
        &state.pool,
        user.0,
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
    user: CurrentUser,
) -> Result<HttpResponse, ApiError> {
    let response = services::research(
        &state.pool,
        user.0,
    )
    .await?;

    Ok(
        HttpResponse::Ok()
            .json(response),
    )
}

pub async fn start_research(
    state: web::Data<AppState>,
    user: CurrentUser,
    input: web::Json<ResearchStartRequest>,
) -> Result<HttpResponse, ApiError> {
    let response = services::start_research(
        &state.pool,
        user.0,
        input.into_inner(),
    )
    .await?;

    Ok(
        HttpResponse::Created()
            .json(response),
    )
}
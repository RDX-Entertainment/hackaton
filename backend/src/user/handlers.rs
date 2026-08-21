use actix_web::{
    web,
    HttpResponse,
};

use crate::{
    auth::CurrentUser,
    error::ApiError,
    AppState,
};

use super::{
    dtos::{
        CertificationStartRequest,
        CreateCompromisedListEntryRequest,
        ProfileUpdate,
        SettingsUpdate,
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
        // Dashboard
        //
        .route(
            "/dashboard",
            web::get().to(dashboard),
        )

        //
        // Profile
        //
        .route(
            "/profile",
            web::get().to(profile),
        )
        .route(
            "/profile",
            web::put().to(save_profile),
        )

        //
        // Compromised Lists
        //
        .route(
            "/lists",
            web::get().to(compromised_lists),
        )
        .route(
            "/lists",
            web::post().to(add_compromised_list),
        )
        .route(
            "/lists/{id}",
            web::delete().to(delete_compromised_list),
        )

        //
        // Certifications
        //
        .route(
            "/certifications",
            web::get().to(certifications),
        )
        .route(
            "/certifications/{level}/start",
            web::post().to(start_certification),
        )
        .route(
            "/certifications/{level}/complete",
            web::post().to(complete_certification),
        )

        //
        // Logs
        //
        .route(
            "/logs",
            web::get().to(player_log),
        )

        //
        // Settings
        //
        .route(
            "/settings",
            web::get().to(settings),
        )
        .route(
            "/settings",
            web::put().to(save_settings),
        );
}

//
// ==============================
// DASHBOARD
// ==============================
//

pub async fn dashboard(
    state: web::Data<AppState>,
    user: CurrentUser,
) -> Result<HttpResponse, ApiError> {
    //
    // IMPORTANTE:
    //
    // Tu código original llamaba:
    //
    // actions()
    //
    // pero no incluiste esa función.
    //
    // Cuando me pases su definición,
    // sustituimos esto.
    //

    let actions =
        serde_json::Value::Array(Vec::new());

    let response = services::dashboard(
        &state.pool,
        user.0,
        actions,
    )
    .await?;

    Ok(
        HttpResponse::Ok()
            .json(response),
    )
}

//
// ==============================
// LOGS
// ==============================
//

pub async fn player_log(
    state: web::Data<AppState>,
    user: CurrentUser,
) -> Result<HttpResponse, ApiError> {
    let response = services::player_log(
        &state.pool,
        user.0,
    )
    .await?;

    Ok(
        HttpResponse::Ok()
            .json(response),
    )
}

//
// ==============================
// SETTINGS
// ==============================
//

pub async fn settings(
    state: web::Data<AppState>,
    user: CurrentUser,
) -> Result<HttpResponse, ApiError> {
    let response = services::settings(
        &state.pool,
        user.0,
    )
    .await?;

    Ok(
        HttpResponse::Ok()
            .json(response),
    )
}

pub async fn save_settings(
    state: web::Data<AppState>,
    user: CurrentUser,
    input: web::Json<SettingsUpdate>,
) -> Result<HttpResponse, ApiError> {
    let response = services::save_settings(
        &state.pool,
        user.0,
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
// CERTIFICATIONS
// ==============================
//

pub async fn certifications(
    state: web::Data<AppState>,
    user: CurrentUser,
) -> Result<HttpResponse, ApiError> {
    let response = services::certifications(
        &state.pool,
        user.0,
    )
    .await?;

    Ok(
        HttpResponse::Ok()
            .json(response),
    )
}

pub async fn start_certification(
    state: web::Data<AppState>,
    user: CurrentUser,
    level: web::Path<i16>,
    input: web::Json<CertificationStartRequest>,
) -> Result<HttpResponse, ApiError> {
    let response = services::start_certification(
        &state.pool,
        user.0,
        level.into_inner(),
        input.into_inner(),
    )
    .await?;

    Ok(
        HttpResponse::Ok()
            .json(response),
    )
}

pub async fn complete_certification(
    state: web::Data<AppState>,
    user: CurrentUser,
    level: web::Path<i16>,
) -> Result<HttpResponse, ApiError> {
    let response = services::complete_certification(
        &state.pool,
        user.0,
        level.into_inner(),
    )
    .await?;

    Ok(
        HttpResponse::Ok()
            .json(response),
    )
}

//
// ==============================
// PROFILE
// ==============================
//

pub async fn profile(
    state: web::Data<AppState>,
    user: CurrentUser,
) -> Result<HttpResponse, ApiError> {
    let response = services::profile(
        &state.pool,
        user.0,
    )
    .await?;

    Ok(
        HttpResponse::Ok()
            .json(response),
    )
}

pub async fn save_profile(
    state: web::Data<AppState>,
    user: CurrentUser,
    input: web::Json<ProfileUpdate>,
) -> Result<HttpResponse, ApiError> {
    let response = services::save_profile(
        &state.pool,
        user.0,
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
// COMPROMISED LISTS
// ==============================
//

pub async fn compromised_lists(
    state: web::Data<AppState>,
    user: CurrentUser,
) -> Result<HttpResponse, ApiError> {
    let response =
        services::compromised_lists(
            &state.pool,
            user.0,
        )
        .await?;

    Ok(
        HttpResponse::Ok()
            .json(response),
    )
}

pub async fn add_compromised_list(
    state: web::Data<AppState>,
    user: CurrentUser,
    input: web::Json<CreateCompromisedListEntryRequest>,
) -> Result<HttpResponse, ApiError> {
    let response =
        services::add_compromised_list(
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

pub async fn delete_compromised_list(
    state: web::Data<AppState>,
    user: CurrentUser,
    id: web::Path<i64>,
) -> Result<HttpResponse, ApiError> {
    services::delete_compromised_list(
        &state.pool,
        user.0,
        id.into_inner(),
    )
    .await?;

    Ok(
        HttpResponse::NoContent()
            .finish(),
    )
}
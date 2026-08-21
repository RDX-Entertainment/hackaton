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
    dtos::ComposeMailRequest,
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
            "/mails",
            web::get().to(mails),
        )
        .route(
            "/mails",
            web::post().to(compose_mail),
        )
        .route(
            "/mails/{id}",
            web::get().to(mail),
        )
        .route(
            "/mails/{id}",
            web::delete().to(delete_mail),
        );
}

//
// ==============================
// INBOX
// ==============================
//

pub async fn mails(
    state: web::Data<AppState>,
    user: CurrentUser,
) -> Result<HttpResponse, ApiError> {
    let response = services::inbox(
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
// READ
// ==============================
//

pub async fn mail(
    state: web::Data<AppState>,
    user: CurrentUser,
    id: web::Path<i64>,
) -> Result<HttpResponse, ApiError> {
    let response = services::read(
        &state.pool,
        user.0,
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
// COMPOSE
// ==============================
//

pub async fn compose_mail(
    state: web::Data<AppState>,
    user: CurrentUser,
    input: web::Json<ComposeMailRequest>,
) -> Result<HttpResponse, ApiError> {
    let response = services::compose(
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
// DELETE
// ==============================
//

pub async fn delete_mail(
    state: web::Data<AppState>,
    user: CurrentUser,
    id: web::Path<i64>,
) -> Result<HttpResponse, ApiError> {
    services::delete(
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

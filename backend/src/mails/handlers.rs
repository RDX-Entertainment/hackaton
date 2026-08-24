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
    ctx: Ctx,
) -> Result<HttpResponse, ApiError> {
    let response = services::inbox(
        &state.pool,
        ctx.id,
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
    ctx: Ctx,
    id: web::Path<i64>,
) -> Result<HttpResponse, ApiError> {
    let response = services::read(
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
// COMPOSE
// ==============================
//

pub async fn compose_mail(
    state: web::Data<AppState>,
    ctx: Ctx,
    input: web::Json<ComposeMailRequest>,
) -> Result<HttpResponse, ApiError> {
    let response = services::compose(
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
// DELETE
// ==============================
//

pub async fn delete_mail(
    state: web::Data<AppState>,
    ctx: Ctx,
    id: web::Path<i64>,
) -> Result<HttpResponse, ApiError> {
    services::delete(
        &state.pool,
        ctx.id,
        id.into_inner(),
    )
    .await?;

    Ok(
        HttpResponse::NoContent()
            .finish(),
    )
}

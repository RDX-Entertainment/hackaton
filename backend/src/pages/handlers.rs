use actix_web::{
    web,
    HttpResponse,
};

use crate::{
    auth::Ctx,
    error::ApiError,
};

use super::services;

pub fn configure(
    cfg: &mut web::ServiceConfig,
) {
    cfg.route(
        "/pages/{page}",
        web::get().to(page_data),
    );
}

pub async fn page_data(
    ctx: Ctx,
    page: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let response =
        services::page_data(
            ctx.id,
            page.into_inner(),
        )?;

    Ok(HttpResponse::Ok().json(response))
}
use actix_web::{
    web,
    HttpResponse,
};

use crate::{
    auth::CurrentUser,
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
    user: CurrentUser,
    page: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let response =
        services::page_data(
            user.0,
            page.into_inner(),
        )?;

    Ok(HttpResponse::Ok().json(response))
}
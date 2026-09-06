use actix_governor::governor::middleware::NoOpMiddleware;
use actix_governor::{Governor, GovernorConfig};
use actix_web::{
    web,
    HttpRequest,
    HttpResponse,
};

use crate::{
    error::ApiError,
    state::AppState,
};

use super::{
    dtos::{LanguageRequest, LoginRequest, RegisterRequest},
    middleware::Ctx,
    ratelimit::ClientIp,
    services,
};

//
// ==============================
// ROUTES
// ==============================
//

/// Las dos rutas que se pueden llamar sin haber entrado llevan el limite de intentos
/// encima; `/me` no, porque ya exige un token.
pub fn configure(
    cfg: &mut web::ServiceConfig,
    attempts: &GovernorConfig<ClientIp, NoOpMiddleware>,
) {
    cfg
        .service(
            web::resource("/auth/login")
                .wrap(Governor::new(attempts))
                .route(web::post().to(login)),
        )
        .service(
            web::resource("/auth/register")
                .wrap(Governor::new(attempts))
                .route(web::post().to(register)),
        )
        .route(
            "/me",
            web::get().to(me),
        )
        .route(
            "/me/language",
            web::put().to(set_language),
        );
}

//
// ==============================
// LOGIN
// ==============================
//

pub async fn login(
    request: HttpRequest,
    state: web::Data<AppState>,
    input: web::Json<LoginRequest>,
) -> Result<HttpResponse, ApiError> {
    let input = input.into_inner();

    // Para la bitacora, no para la respuesta: al que llama se le sigue contestando lo
    // mismo tanto si el nombre no existe como si la contrasena no es esa. Decirle cual de
    // las dos cosa falla es regalarle la mitad del trabajo.
    let identifier = input.username.trim().to_owned();

    let response = services::login(
        &state.pool,
        &state.jwt_secret,
        input,
    )
    .await
    .inspect_err(|error| {
        if matches!(error, ApiError::Unauthorized) {
            let origin = request
                .connection_info()
                .realip_remote_addr()
                .unwrap_or("desconocido")
                .to_owned();

            tracing::warn!(
                identifier = %identifier,
                origin = %origin,
                "intento de entrada fallido",
            );
        }
    })?;

    Ok(
        HttpResponse::Ok()
            .json(response),
    )
}

pub async fn set_language(
    state: web::Data<AppState>,
    ctx: Ctx,
    input: web::Json<LanguageRequest>,
) -> Result<HttpResponse, ApiError> {
    let response = services::set_language(
        &state.pool,
        ctx.id,
        &input.into_inner().lang,
    )
    .await?;

    Ok(
        HttpResponse::Ok()
            .json(response),
    )
}

//
// ==============================
// REGISTER
// ==============================
//

pub async fn register(
    state: web::Data<AppState>,
    input: web::Json<RegisterRequest>,
) -> Result<HttpResponse, ApiError> {
    let response = services::register(
        &state.pool,
        &state.jwt_secret,
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
// CURRENT USER
// ==============================
//

pub async fn me(
    state: web::Data<AppState>,
    ctx: Ctx,
) -> Result<HttpResponse, ApiError> {
    let response = services::me(
        &state.pool,
        ctx.id,
    )
    .await?;

    Ok(
        HttpResponse::Ok()
            .json(response),
    )
}

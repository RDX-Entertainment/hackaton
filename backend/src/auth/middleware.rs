//! El contexto de quien hace la peticion, y la puerta que separa jugadores de
//! administradores.
//!
//! Hay dos formas de leer ese contexto y comparten resolucion:
//!
//!   * `Ctx`       extractor. Cualquier handler que lo pida en su firma queda cerrado a
//!                 quien no traiga token, y ademas sabe si quien llama es administrador.
//!   * `AdminOnly` middleware. Envuelve un scope entero y decide antes de que la peticion
//!                 llegue a ningun handler.
//!
//! El middleware responde **404 con el cuerpo vacio**, no 401 ni 403. Es deliberado: un
//! 403 confirma que la ruta existe y que solo falta permiso, que es justo lo que no
//! interesa contarle a quien va probando. Con 404 vacio, `/api/admin/parts` es
//! indistinguible de `/api/noexiste` para cualquiera que no sea administrador.

use std::rc::Rc;

use actix_web::{
    body::EitherBody,
    dev::{forward_ready, Payload, Service, ServiceRequest, ServiceResponse, Transform},
    http::header::{self, HeaderMap},
    web, Error, FromRequest, HttpMessage, HttpRequest, HttpResponse,
};
use futures_util::future::{ready, LocalBoxFuture, Ready};
use jsonwebtoken::{decode, DecodingKey, Validation};

use crate::{error::ApiError, state::AppState};

use super::models::Claims;

//
// ==============================
// CONTEXT
// ==============================
//

/// Quien esta llamando. Se resuelve una vez por peticion y viaja en las extensiones.
#[derive(Debug, Clone, Copy)]
pub struct Ctx {
    pub id: i64,
    pub is_admin: bool,
}

impl Ctx {
    /// Falla con NotFound, no con Forbidden, por la misma razon que el middleware: negar
    /// el permiso en voz alta es confirmar que hay algo detras.
    pub fn require_admin(&self) -> Result<(), ApiError> {
        if self.is_admin {
            return Ok(());
        }

        Err(ApiError::NotFound)
    }
}

//
// ==============================
// RESOLUTION
// ==============================
//

fn bearer(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
}

fn user_from_token(token: &str, secret: &str) -> Option<i64> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .ok()
    .map(|data| data.claims.sub)
}

/// Token valido mas la marca de administrador, leida en el momento.
///
/// Se consulta en cada peticion en vez de guardarla en el token: un administrador al que
/// se le retira el permiso lo pierde ya, y no cuando le caduque la sesion.
async fn load(state: &AppState, headers: &HeaderMap) -> Option<Ctx> {
    let token = bearer(headers)?;
    let id = user_from_token(token, &state.jwt_secret)?;

    let is_admin = sqlx::query_scalar::<_, bool>("SELECT is_admin FROM users WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.pool)
        .await
        .ok()??;

    Some(Ctx { id, is_admin })
}

impl FromRequest for Ctx {
    type Error = ApiError;
    type Future = LocalBoxFuture<'static, Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        // Si el middleware ya lo resolvio, no se vuelve a consultar la base.
        if let Some(ctx) = req.extensions().get::<Ctx>().copied() {
            return Box::pin(async move { Ok(ctx) });
        }

        let state = req.app_data::<web::Data<AppState>>().cloned();
        let headers = req.headers().clone();

        Box::pin(async move {
            let state = state.ok_or(ApiError::Unauthorized)?;

            load(&state, &headers)
                .await
                .ok_or(ApiError::Unauthorized)
        })
    }
}

//
// ==============================
// ADMIN GATE
// ==============================
//

/// Envuelve un scope y lo hace invisible para quien no sea administrador.
pub struct AdminOnly;

impl<S, B> Transform<S, ServiceRequest> for AdminOnly
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = AdminOnlyGate<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AdminOnlyGate {
            service: Rc::new(service),
        }))
    }
}

pub struct AdminOnlyGate<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for AdminOnlyGate<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = Rc::clone(&self.service);

        Box::pin(async move {
            let state = req.app_data::<web::Data<AppState>>().cloned();

            let ctx = match state {
                Some(state) => load(&state, req.headers()).await,
                None => None,
            };

            match ctx {
                Some(ctx) if ctx.is_admin => {
                    // Resuelto aqui, para que el handler no repita la consulta.
                    req.extensions_mut().insert(ctx);

                    service
                        .call(req)
                        .await
                        .map(ServiceResponse::map_into_left_body)
                }
                _ => {
                    // Cuerpo vacio a proposito: es exactamente lo que devuelve actix
                    // cuando la ruta no existe.
                    let (request, _) = req.into_parts();

                    Ok(ServiceResponse::new(
                        request,
                        HttpResponse::NotFound().finish().map_into_right_body(),
                    ))
                }
            }
        })
    }
}

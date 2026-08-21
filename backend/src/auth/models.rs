use actix_web::{dev::Payload, http::header, FromRequest, HttpRequest};
use futures_util::future::{ready, Ready};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::{error::ApiError, state::AppState};

//
// ==============================
// TOKEN
// ==============================
//

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i64,
    pub exp: usize,
}

/// El id del jugador autenticado, sacado del Bearer token. Al implementar FromRequest,
/// cualquier handler que lo pida en su firma queda cerrado a quien no traiga token.
pub struct CurrentUser(pub i64);

impl FromRequest for CurrentUser {
    type Error = ApiError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(
        req: &HttpRequest,
        _: &mut Payload,
    ) -> Self::Future {
        let state = match req
            .app_data::<actix_web::web::Data<AppState>>()
        {
            Some(state) => state,
            None => {
                return ready(Err(
                    ApiError::Unauthorized,
                ))
            }
        };

        let token = req
            .headers()
            .get(header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| {
                value.strip_prefix("Bearer ")
            });

        let result = token
            .and_then(|token| {
                decode::<Claims>(
                    token,
                    &DecodingKey::from_secret(
                        state.jwt_secret.as_bytes(),
                    ),
                    &Validation::default(),
                )
                .ok()
            })
            .map(|data| {
                CurrentUser(data.claims.sub)
            })
            .ok_or(ApiError::Unauthorized);

        ready(result)
    }
}

//
// ==============================
// ROWS
// ==============================
//

#[derive(Debug, Clone, FromRow)]
pub struct UserRow {
    pub id: i64,
    pub login: String,
    pub email: String,
    pub game_ip: i64,
    pub premium: bool,
}

/// La fila del login lleva ademas el hash, que nunca sale del modulo.
#[derive(Debug, Clone, FromRow)]
pub struct CredentialsRow {
    pub id: i64,
    pub login: String,
    pub email: String,
    pub game_ip: i64,
    pub premium: bool,
    pub password: String,
}

use actix_web::{FromRequest, HttpRequest, dev::Payload, http::header};
use futures_util::future::{Ready, ready};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

use crate::{error::ApiError, state::AppState};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i64,
    pub exp: usize,
}
pub struct CurrentUser(pub i64);

impl FromRequest for CurrentUser {
    type Error = ApiError;
    type Future = Ready<Result<Self, Self::Error>>;
    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let state = match req.app_data::<actix_web::web::Data<AppState>>() {
            Some(s) => s,
            None => return ready(Err(ApiError::Unauthorized)),
        };
        let token = req
            .headers()
            .get(header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "));
        let result = token
            .and_then(|t| {
                decode::<Claims>(
                    t,
                    &DecodingKey::from_secret(state.jwt_secret.as_bytes()),
                    &Validation::default(),
                )
                .ok()
            })
            .map(|data| CurrentUser(data.claims.sub))
            .ok_or(ApiError::Unauthorized);
        ready(result)
    }
}

pub fn issue_token(user_id: i64, secret: &str) -> Result<String, ApiError> {
    let exp = (chrono::Utc::now() + chrono::Duration::days(7)).timestamp() as usize;
    encode(
        &Header::default(),
        &Claims { sub: user_id, exp },
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|_| ApiError::Unauthorized)
}

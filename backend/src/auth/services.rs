use bcrypt::{hash, verify};
use jsonwebtoken::{encode, EncodingKey, Header};
use rand::{distr::Alphanumeric, Rng};
use sqlx::PgPool;

use crate::error::ApiError;

use super::{
    dtos::{
        LoginRequest,
        RegisterRequest,
        SessionResponse,
        UserResponse,
    },
    models::Claims,
    repo,
};

//
// ==============================
// TOKEN
// ==============================
//

/// Una semana de validez, como venia del legado.
const TOKEN_DAYS: i64 = 7;

pub fn issue_token(
    user_id: i64,
    secret: &str,
) -> Result<String, ApiError> {
    let exp = (chrono::Utc::now()
        + chrono::Duration::days(TOKEN_DAYS))
    .timestamp() as usize;

    encode(
        &Header::default(),
        &Claims {
            sub: user_id,
            exp,
        },
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|_| ApiError::Unauthorized)
}

//
// ==============================
// LOGIN
// ==============================
//

pub async fn login(
    pool: &PgPool,
    jwt_secret: &str,
    input: LoginRequest,
) -> Result<SessionResponse, ApiError> {
    let credentials =
        repo::find_credentials_by_login(
            pool,
            input.username.trim(),
        )
        .await?
        .ok_or(ApiError::Unauthorized)?;

    let matches = verify(
        &input.password,
        &credentials.password,
    )
    .map_err(|_| ApiError::Unauthorized)?;

    if !matches {
        return Err(ApiError::Unauthorized);
    }

    let id = credentials.id;

    repo::touch_last_login(pool, id).await?;

    Ok(SessionResponse {
        token: issue_token(id, jwt_secret)?,
        user: credentials.into(),
    })
}

//
// ==============================
// REGISTER
// ==============================
//

/// Coste de bcrypt. Trece es lo que traia el port.
const BCRYPT_COST: u32 = 13;

/// Longitud de la contrasena de maquina que se le asigna al jugador.
const GAME_PASS_LEN: usize = 8;

pub async fn register(
    pool: &PgPool,
    jwt_secret: &str,
    input: RegisterRequest,
) -> Result<SessionResponse, ApiError> {
    let name = input.username.trim();

    if !valid_name(name) {
        return Err(ApiError::BadRequest(
            "invalid username".into(),
        ));
    }

    if input.password.chars().count()
        < PASSWORD_MIN
    {
        return Err(ApiError::BadRequest(
            "password must be at least 8 characters"
                .into(),
        ));
    }

    // Quien no da correo recibe uno interno. La columna es NOT NULL UNIQUE y el nombre
    // de jugador ya es unico, asi que derivarlo del nombre basta y no colisiona.
    let email = match input
        .email
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        Some(value) => value.to_owned(),
        None => format!("{name}@local"),
    };

    let taken = repo::login_or_email_taken(
        pool,
        name,
        &email,
    )
    .await?;

    if taken {
        return Err(ApiError::BadRequest(
            "username or email already exists"
                .into(),
        ));
    }

    let game_ip = rand::rng()
        .random_range(1_i64..=u32::MAX as i64);

    let game_pass: String = rand::rng()
        .sample_iter(Alphanumeric)
        .take(GAME_PASS_LEN)
        .map(char::from)
        .collect();

    let password_hash =
        hash(&input.password, BCRYPT_COST)
            .map_err(|_| {
                ApiError::BadRequest(
                    "cannot hash password".into(),
                )
            })?;

    let mut tx = pool.begin().await?;

    let id = repo::insert_user(
        &mut tx,
        name,
        &password_hash,
        &email,
        &game_pass,
        game_ip,
    )
    .await?;

    repo::bootstrap_player(&mut tx, id).await?;

    let user =
        repo::find_user_in_tx(&mut tx, id).await?;

    tx.commit().await?;

    Ok(SessionResponse {
        token: issue_token(id, jwt_secret)?,
        user: user.into(),
    })
}

//
// ==============================
// CURRENT USER
// ==============================
//

pub async fn me(
    pool: &PgPool,
    user_id: i64,
) -> Result<UserResponse, ApiError> {
    let user =
        repo::find_user_by_id(pool, user_id)
            .await?
            .ok_or(ApiError::NotFound)?;

    Ok(user.into())
}

//
// ==============================
// VALIDATION
// ==============================
//

/// Lo unico que se le pide a una contrasena. Se cuentan caracteres y no bytes, para que
/// una clave con acentos o emoji no cuente de mas.
const PASSWORD_MIN: usize = 8;

/// El nombre sigue con las reglas del legado: hasta quince caracteres, alfanumericos mas
/// punto, guion y guion bajo. Es lo que se pinta en logs y perfiles.
fn valid_name(name: &str) -> bool {
    (1..=15).contains(&name.len())
        && name.bytes().all(|byte| {
            byte.is_ascii_alphanumeric()
                || matches!(
                    byte,
                    b'.' | b'_' | b'-'
                )
        })
}

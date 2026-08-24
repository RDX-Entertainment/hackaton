use serde::{Deserialize, Serialize};
use sqlx::FromRow;

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

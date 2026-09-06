use serde::{Deserialize, Serialize};

use super::models::{CredentialsRow, UserRow};

//
// ==============================
// REQUESTS
// ==============================
//

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    // Nombre de usuario o correo, indiferentemente. El campo conserva el nombre
    // `username` para no romper el contrato con clientes ya escritos.
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,

    // Opcional a proposito: registrarse no depende de tener correo, ni hay nada que
    // mandar a ninguna parte. Si no viene, el modulo se inventa uno interno.
    #[serde(default)]
    pub email: Option<String>,
}

//
// ==============================
// RESPONSES
// ==============================
//

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: i64,
    pub login: String,
    pub email: String,
    pub game_ip: i64,
    pub premium: bool,
    pub lang: String,
}

impl From<UserRow> for UserResponse {
    fn from(value: UserRow) -> Self {
        Self {
            id: value.id,
            login: value.login,
            email: value.email,
            game_ip: value.game_ip,
            premium: value.premium,
            lang: value.lang,
        }
    }
}

/// El hash se queda fuera: la fila de credenciales entra, la respuesta sale sin el.
impl From<CredentialsRow> for UserResponse {
    fn from(value: CredentialsRow) -> Self {
        Self {
            id: value.id,
            login: value.login,
            email: value.email,
            game_ip: value.game_ip,
            premium: value.premium,
            lang: value.lang,
        }
    }
}

/// La lengua en la que quiere leer el jugador.
#[derive(Debug, Deserialize)]
pub struct LanguageRequest {
    pub lang: String,
}

#[derive(Debug, Serialize)]
pub struct SessionResponse {
    pub token: String,
    pub user: UserResponse,
}

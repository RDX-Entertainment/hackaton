use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::models::{
    DashboardUserRow,
    AdminUserRow,
    ProfileRow,
    SettingsRow,
};

//
// ==============================
// DASHBOARD
// ==============================
//

#[derive(Debug, Serialize)]
pub struct DashboardUserResponse {
    pub id: i64,
    pub login: String,
    pub email: String,
    pub game_ip: i64,
    pub premium: bool,
}

impl From<DashboardUserRow> for DashboardUserResponse {
    fn from(value: DashboardUserRow) -> Self {
        Self {
            id: value.id,
            login: value.login,
            email: value.email,
            game_ip: value.game_ip,
            premium: value.premium,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct DashboardResponse {
    pub user: DashboardUserResponse,
    pub money: f64,
    pub processes: i64,
    pub unread_mail: i64,

    // Temporal porque no pasaste la definición de actions()
    pub actions: Value,
}

//
// ==============================
// SETTINGS
// ==============================
//

#[derive(Debug, Serialize)]
pub struct SettingsResponse {
    pub login: String,
    pub email: String,
    pub premium: bool,
    pub last_login: DateTime<Utc>,
}

impl From<SettingsRow> for SettingsResponse {
    fn from(value: SettingsRow) -> Self {
        Self {
            login: value.login,
            email: value.email,
            premium: value.premium,
            last_login: value.last_login,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct SettingsUpdate {
    pub current_password: String,
    pub email: Option<String>,
    pub new_password: Option<String>,
}

//
// ==============================
// GENERIC
// ==============================
//

#[derive(Debug, Serialize)]
pub struct SavedResponse {
    pub saved: bool,
}

#[derive(Debug, Serialize)]
pub struct CreatedIdResponse {
    pub id: i64,
}

//
// ==============================
// CERTIFICATIONS
// ==============================
//

#[derive(Debug, Serialize)]
pub struct CertificationItemResponse {
    pub level: i16,
    pub name: &'static str,
    pub description: &'static str,
    pub price: i64,
    pub completed: bool,
    pub available: bool,
    pub learning: bool,
}

#[derive(Debug, Serialize)]
pub struct CertificationsResponse {
    pub level: i16,
    pub items: Vec<CertificationItemResponse>,
}

#[derive(Debug, Deserialize)]
pub struct CertificationStartRequest {
    pub account: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct CertificationStartedResponse {
    pub level: i16,
    pub started: bool,
}

#[derive(Debug, Serialize)]
pub struct CertificationCompletedResponse {
    pub level: i16,
    pub completed: bool,
}

//
// ==============================
// PROFILE
// ==============================
//

#[derive(Debug, Serialize)]
pub struct ProfileResponse {
    pub id: i64,
    pub login: String,
    pub game_ip: i64,
    pub biography: String,

    /// La pagina que sirve su maquina. Se pinta en un marco aislado, nunca en linea.
    pub web: String,
    pub rank: i32,
    pub hacks: i32,
    pub money_hardware: i64,
    pub exp: i64,
    pub date_joined: Option<DateTime<Utc>>,
}

impl From<ProfileRow> for ProfileResponse {
    fn from(value: ProfileRow) -> Self {
        Self {
            id: value.id,
            login: value.login,
            game_ip: value.game_ip,
            biography: value.biography,
            web: value.web,
            rank: value.rank,
            hacks: value.hack_count,
            money_hardware: value.money_hardware,
            exp: value.exp,
            date_joined: value.date_joined,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ProfileUpdate {
    pub biography: String,

    /// Opcional: guardar solo la biografia no borra la pagina.
    #[serde(default)]
    pub web: Option<String>,
}

//
// ==============================
// COMPROMISED LISTS
// ==============================
//

#[derive(Debug, Deserialize)]
pub struct CreateCompromisedListEntryRequest {
    pub ip: i64,
    pub login: String,
    pub password: String,
    pub virus_id: Option<i64>,
}
//
// ==============================
// ADMIN
// ==============================
//

#[derive(Debug, Serialize)]
pub struct AdminUserResponse {
    pub id: i64,
    pub login: String,
    pub email: String,
    pub game_ip: i64,
    pub is_admin: bool,
    pub premium: bool,
    pub last_login: DateTime<Utc>,
    pub components: i64,
}

impl From<AdminUserRow> for AdminUserResponse {
    fn from(value: AdminUserRow) -> Self {
        Self {
            id: value.id,
            login: value.login,
            email: value.email,
            game_ip: value.game_ip,
            is_admin: value.is_admin,
            premium: value.premium,
            last_login: value.last_login,
            components: value.components,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct AdminFlagUpdate {
    pub admin: bool,
}

/// Alta desde el panel. El kit es opcional: sin él, la cuenta conserva el kit por defecto.
#[derive(Debug, Deserialize)]
pub struct AdminCreateUserRequest {
    pub username: String,
    pub email: Option<String>,
    pub password: String,
    pub kit_id: Option<i64>,
    #[serde(default)]
    pub admin: bool,
}

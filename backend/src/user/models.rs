use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct DashboardUserRow {
    pub id: i64,
    pub login: String,
    pub email: String,
    pub game_ip: i64,
    pub premium: bool,
}

#[derive(Debug, Clone, FromRow)]
pub struct SettingsRow {
    pub login: String,
    pub email: String,
    pub premium: bool,
    pub last_login: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
// Solo lo que el juego mantiene de verdad.
//
// users_stats tiene ocho contadores mas -- mission_count, money_earned, money_transfered,
// money_research, ddos_count, profile_views, time_playing -- y cache tiene reputation.
// Ninguno tiene una sola escritura en todo el backend: son columnas del esquema heredado
// que este port aun no ha cableado. Se quedan en la tabla, que es donde aterrizaran cuando
// se porten, pero fuera de aqui: devolver un cero permanente con nombre de estadistica es
// mentir con mas pasos.
pub struct ProfileRow {
    pub id: i64,
    pub login: String,
    pub game_ip: i64,
    pub biography: String,
    pub web: String,
    pub rank: i32,
    pub hack_count: i32,
    pub money_hardware: i64,
    pub exp: i64,

    // Los contadores valen 0 si falta la fila del LEFT JOIN, pero una fecha de alta
    // inventada mentiria, asi que va opcional.
    pub date_joined: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct PlayerLogEntry {
    pub id: i64,
    pub text: String,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct CompromisedListEntry {
    pub id: i64,
    pub ip: i64,
    pub login: String,
    pub hacked_at: DateTime<Utc>,
    pub virus_id: i64,
}

#[derive(Debug, Clone, Copy)]
pub struct CertificationDefinition {
    pub level: i16,
    pub name: &'static str,
    pub description: &'static str,
    pub price: i64,
}

impl CertificationDefinition {
    /// El hecho que la aprueba, cuando se aprueba haciendo.
    ///
    /// Los dos primeros niveles **son el tutorial**: el 1 es compilar el primer programa y el
    /// 2 es entrar en la primera maquina. Los demas se estudian y se pagan, que es lo que
    /// hacian los cinco.
    pub const fn earned_by(level: i16) -> Option<&'static str> {
        match level {
            1 => Some("compile"),
            2 => Some("intrusion"),
            _ => None,
        }
    }
}

impl CertificationDefinition {
    pub fn by_level(level: i16) -> Option<Self> {
        match level {
            1 => Some(Self {
                level,
                name: "Basic tutorial",
                description: "Follow this tutorial to learn how to play",
                price: 0,
            }),

            2 => Some(Self {
                level,
                name: "Hacking 101",
                description: "Learn the basics of hacking",
                price: 0,
            }),

            3 => Some(Self {
                level,
                name: "Intermediate hacking",
                description: "Improve your hacking techniques",
                price: 50,
            }),

            4 => Some(Self {
                level,
                name: "Advanced hacking",
                description: "Meet the wonders of DDoS world",
                price: 200,
            }),

            5 => Some(Self {
                level,
                name: "DDoS security",
                description: "Learn to protect yourself from DDoS",
                price: 500,
            }),

            _ => None,
        }
    }

    pub fn all() -> impl Iterator<Item = Self> {
        (1..=5).filter_map(Self::by_level)
    }
}
/// Una linea del listado del panel. Ademas de la cuenta lleva cuantas piezas tiene, que es
/// lo que un administrador quiere saber antes de regalarle mas.
#[derive(Debug, Clone, FromRow)]
pub struct AdminUserRow {
    pub id: i64,
    pub login: String,
    pub email: String,
    pub game_ip: i64,
    pub is_admin: bool,
    pub premium: bool,
    pub last_login: DateTime<Utc>,
    pub components: i64,
}

use chrono::{DateTime, Utc};
use sqlx::FromRow;

//
// ==============================
// RANKING
// ==============================
//

#[derive(Debug, Clone, FromRow)]
pub struct RankingRow {
    pub rank: i32,
    pub id: i64,
    pub login: String,
}

//
// ==============================
// FRIENDS
// ==============================
//

#[derive(Debug, Clone, FromRow)]
pub struct FriendRow {
    pub id: i64,
    pub login: String,
}

//
// ==============================
// CLANS
// ==============================
//

#[derive(Debug, Clone, FromRow)]
pub struct ClanRow {
    pub id: i64,
    pub clan_ip: i64,
    pub name: String,
    pub nick: String,
    pub description: String,
    pub slots_total: i16,
    pub members: i64,
    pub money: i64,
    pub power: i64,
}

/// El clan propio trae ademas el rango del jugador dentro de el, que es lo que decide
/// quien puede revisar solicitudes.
#[derive(Debug, Clone, FromRow)]
pub struct MyClanRow {
    pub id: i64,
    pub name: String,
    pub nick: String,
    pub description: String,
    pub slots_total: i16,
    pub money: i64,
    pub power: i64,
    pub auth_level: i16,
}

#[derive(Debug, Clone, FromRow)]
pub struct ClanMemberRow {
    pub id: i64,
    pub login: String,
    pub auth_level: i16,
}

#[derive(Debug, Clone, FromRow)]
pub struct ClanRequestRow {
    pub id: i64,
    pub login: String,
    pub message: String,
    pub created_at: DateTime<Utc>,
}

/// Lo que hace falta para resolver una solicitud: a que clan va, quien la pidio y cuantas
/// plazas tiene el clan.
#[derive(Debug, Clone, FromRow)]
pub struct PendingRequestRow {
    pub clan_id: i64,
    pub user_id: i64,
    pub slots_total: i16,
}

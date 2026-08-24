use chrono::{DateTime, Utc};
use sqlx::FromRow;

#[derive(Debug, Clone)]
pub struct Remote {
    pub is_npc: bool,
    pub id: i64,
    pub name: String,
    pub npc_type: i16,
    pub down: bool,

    /// La pagina que sirve, si sirve alguna. Los jugadores no tienen.
    pub web: Option<String>,
}

#[derive(Debug, Clone, FromRow)]
pub struct NpcRemoteRow {
    pub id: i64,
    pub npc_type: i16,
    pub down: bool,
    pub name: String,
    pub web: Option<String>,
}

#[derive(Debug, Clone, FromRow)]
pub struct PlayerRemoteRow {
    pub id: i64,
    pub login: String,
    pub web: Option<String>,
}

#[derive(Debug, Clone, FromRow)]
pub struct InternetTargetRow {
    pub id: i64,
    pub npc_ip: i64,
    pub npc_type: i16,
    pub name: Option<String>,
    pub web: Option<String>,
}

#[derive(Debug, Clone, FromRow)]
pub struct AnalyzeResultRow {
    pub cpu_min: i32,
    pub cpu_max: i32,
    pub ram_min: i32,
    pub ram_max: i32,
}

#[derive(Debug, Clone, FromRow)]
pub struct BookmarkRow {
    pub ip: i64,
    pub name: String,
}

#[derive(Debug, Clone, FromRow)]
pub struct HistoryRow {
    pub ip: i64,
    pub name: String,
    pub visited_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct BestAnalyzerRow {
    pub id: i64,
    pub soft_version: i32,
}
/// Un sitio del panel: da igual quien lo sirva, se edita igual.
#[derive(Debug, Clone, FromRow)]
pub struct SiteRow {
    pub kind: String,
    pub id: i64,
    pub ip: i64,
    pub name: String,
    pub web: String,
}

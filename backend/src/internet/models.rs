use chrono::{DateTime, Utc};
use sqlx::FromRow;

#[derive(Debug, Clone)]
pub struct Remote {
    pub is_npc: bool,
    pub id: i64,
    pub name: String,
    pub npc_type: i16,
    pub down: bool,

    /// **La maquina que responde en esa direccion**, que es lo que de verdad hay al otro lado.
    ///
    /// `id` sigue diciendo de quien es —hace falta para casi todo lo que cuelga del dueno: el
    /// ranking, el banco, las evidencias— pero ya no es la respuesta a «que hay en esta IP».
    /// Con varias maquinas por cuenta esas dos preguntas dejaron de tener la misma respuesta.
    ///
    /// `None` solo si la direccion existe sin maquina detras, que la migracion 0049 dejo
    /// imposible para todo lo alcanzable. Se contesta en vez de suponerse.
    pub server_id: Option<i64>,

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
    pub server_id: Option<i64>,
}

#[derive(Debug, Clone, FromRow)]
pub struct PlayerRemoteRow {
    pub id: i64,
    pub login: String,
    pub web: Option<String>,
    pub server_id: Option<i64>,
}

/// Una maquina alcanzada **por dentro**, cruzando un cable desde otra en la que ya se esta.
#[derive(Debug, Clone, FromRow)]
pub struct LanRemoteRow {
    pub server_id: i64,
    pub owner_id: i64,
    pub is_npc: bool,
    pub name: String,
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

/// El banco al que pertenece una maquina, cuando lo es. Lo necesita el navegador para saber
/// donde abre cuenta o contra quien autentica sin que la pagina tenga que decirselo: si lo
/// dijera la pagina, un sitio cualquiera podria hacerse pasar por el banco de al lado.
#[derive(Debug, Clone, FromRow)]
pub struct BankOfMachineRow {
    pub id: i64,
    pub name: String,
}

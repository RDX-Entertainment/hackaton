use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::models::{
    AnalyzeResultRow,
    BankOfMachineRow,
    SiteRow,
    BookmarkRow,
    HistoryRow,
    InternetTargetRow,
};
use super::webscript::Site;

//
// ==============================
// TARGETS
// ==============================
//

#[derive(Debug, Serialize)]
pub struct InternetTargetResponse {
    pub id: i64,
    pub ip: i64,

    #[serde(rename = "type")]
    pub npc_type: i16,

    pub name: Option<String>,
    pub web: Option<String>,
}

impl From<InternetTargetRow> for InternetTargetResponse {
    fn from(value: InternetTargetRow) -> Self {
        Self {
            id: value.id,
            ip: value.npc_ip,
            npc_type: value.npc_type,
            name: value.name,
            web: value.web,
        }
    }
}

//
// ==============================
// NAVIGATION
// ==============================
//

#[derive(Debug, Serialize)]
pub struct HardwareAnalysisResponse {
    pub cpu_min: i32,
    pub cpu_max: i32,
    pub ram_min: i32,
    pub ram_max: i32,
}

impl From<AnalyzeResultRow> for HardwareAnalysisResponse {
    fn from(value: AnalyzeResultRow) -> Self {
        Self {
            cpu_min: value.cpu_min,
            cpu_max: value.cpu_max,
            ram_min: value.ram_min,
            ram_max: value.ram_max,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct InternetNavigateResponse {
    pub ip: i64,
    pub kind: &'static str,
    pub id: i64,
    pub name: String,

    #[serde(rename = "type")]
    pub npc_type: i16,

    pub down: bool,
    pub access: bool,
    pub access_as: Option<String>,
    pub bookmarked: bool,
    pub scanned: bool,
    pub analysis: Option<HardwareAnalysisResponse>,

    /// El HTML que sirve esta maquina. El cliente lo pinta en un marco aislado y sin
    /// permisos: es decoracion, no puede ejecutar nada.
    pub web: Option<String>,

    /// El mismo sitio, ya leido: sus paginas y sus enlaces. Viaja aparte del HTML porque el
    /// marco esta aislado y no puede avisar de un clic, asi que la navegacion la pinta el
    /// navegador del juego por fuera. Va en `None` cuando el sitio no declara nada y basta
    /// con pintar `web` tal cual.
    pub site: Option<Site>,

    /// El banco al que pertenece esta maquina, si es de alguno. Lo dice el servidor y no la
    /// pagina: asi un sitio no puede hacerse pasar por el banco de al lado.
    pub bank: Option<BankRef>,

    /// Las ordenes que esta web tiene permitido usar, salidas de `action_grants`. El
    /// navegador no obedece ninguna que no este aqui, asi que escribir una en el HTML no
    /// sirve de nada si no se ha concedido.
    pub allowed: Vec<String>,
}

/// Lo justo para que el navegador sepa donde esta operando.
#[derive(Debug, Serialize)]
pub struct BankRef {
    pub id: i64,
    pub name: String,
}

impl From<BankOfMachineRow> for BankRef {
    fn from(value: BankOfMachineRow) -> Self {
        Self {
            id: value.id,
            name: value.name,
        }
    }
}

//
// ==============================
// LOGIN
// ==============================
//

#[derive(Debug, Deserialize)]
pub struct InternetLoginRequest {
    pub ip: i64,
    pub user: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct InternetLoginResponse {
    pub access: bool,
    pub name: String,
}

//
// ==============================
// HACK
// ==============================
//

#[derive(Debug, Deserialize)]
pub struct InternetHackRequest {
    pub ip: i64,
    pub kind: String,
}

#[derive(Debug, Serialize)]
pub struct InternetHackResponse {
    pub process_id: i64,
    pub action: i16,
    pub seconds: i32,
    pub attacker_version: i32,
    pub defender_version: i32,
}

//
// ==============================
// IP BODY
// ==============================
//

#[derive(Debug, Deserialize)]
pub struct IpRequest {
    pub ip: i64,
}

//
// ==============================
// ANALYZE
// ==============================
//

#[derive(Debug, Serialize)]
pub struct AnalyzeStartedResponse {
    pub process_id: i64,
    pub seconds: i32,
    pub analyzer_version: i32,
}

//
// ==============================
// LOG
// ==============================
//

#[derive(Debug, Serialize)]
pub struct RemoteLogResponse {
    pub ip: i64,
    pub text: String,
}

#[derive(Debug, Deserialize)]
pub struct LogEditRequest {
    pub text: String,
}

#[derive(Debug, Serialize)]
pub struct LogEditStartedResponse {
    pub process_id: i64,
    pub seconds: i32,
}

//
// ==============================
// BOOKMARKS
// ==============================
//

#[derive(Debug, Serialize)]
pub struct BookmarkResponse {
    pub ip: i64,
    pub name: String,
}

impl From<BookmarkRow> for BookmarkResponse {
    fn from(value: BookmarkRow) -> Self {
        Self {
            ip: value.ip,
            name: value.name,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct BookmarkCreatedResponse {
    pub ip: i64,
}

//
// ==============================
// HISTORY
// ==============================
//

#[derive(Debug, Serialize)]
pub struct InternetHistoryResponse {
    pub ip: i64,
    pub name: String,
    pub visited_at: DateTime<Utc>,
}

impl From<HistoryRow> for InternetHistoryResponse {
    fn from(value: HistoryRow) -> Self {
        Self {
            ip: value.ip,
            name: value.name,
            visited_at: value.visited_at,
        }
    }
}
//
// ==============================
// SITIOS
// ==============================
//

#[derive(Debug, Serialize)]
pub struct SiteResponse {
    pub kind: String,
    pub id: i64,
    pub ip: i64,
    pub name: String,
    pub web: String,
}

impl From<SiteRow> for SiteResponse {
    fn from(value: SiteRow) -> Self {
        Self {
            kind: value.kind,
            id: value.id,
            ip: value.ip,
            name: value.name,
            web: value.web,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct SiteUpdate {
    pub web: String,
}

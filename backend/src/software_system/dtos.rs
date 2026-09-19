use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct Capability {
    pub code: String,
    pub name: String,
    pub domain: String,
    pub role: String,
    pub action_kind: String,
    pub relevant_attributes: Vec<String>,
    pub secondary_attributes: Vec<String>,
    pub available_actions: Vec<String>,
    pub max_access_scope: String,
    pub granted_permissions: Vec<String>,
}
#[derive(Serialize)]
pub struct ResearchModule {
    pub id: i64,
    pub code: String,
    pub name: String,
    pub capability: Option<String>,
    pub deltas: Stats,
}
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct Stats {
    pub effectiveness: i32,
    pub resistance: i32,
    pub stealth: i32,
    pub detection: i32,
    pub removal: i32,
    pub penetration: i32,
    pub hardening: i32,
    pub manipulation: i32,
    pub integrity: i32,
    pub anti_forensics: i32,
    pub forensics: i32,
    pub persistence: i32,
    pub stability: i32,
    pub cpu: i32,
    pub ram: i32,
    pub net: i32,
    pub weight: i32,
}
#[derive(Deserialize)]
pub struct CreateBlueprintRequest {
    pub name: String,
    pub capability: String,
    #[serde(default)]
    pub module_ids: Vec<i64>,
}
#[derive(Serialize)]
pub struct Blueprint {
    pub id: i64,
    pub name: String,
    pub revision: i32,
    pub capability: String,
    pub modules: Vec<i64>,
    pub projected: Stats,
}
#[derive(Deserialize)]
pub struct CompileRequest {
    pub blueprint_id: i64,
}
#[derive(Serialize)]
pub struct CompilationQueued {
    pub process_id: i64,
    pub blueprint_id: i64,
    pub name: String,
    pub capability: String,
    pub seconds: i32,
    pub stats: Stats,
}
/// Una copia compilada, vista desde el arsenal del jugador. `software` la lista como un
/// fichero mas —nombre, tamano, si corre—, y lo que falta ahi es justo lo que decide si
/// sirve para una operacion: que capacidad tiene y con que atributos se compilo.
#[derive(Serialize)]
pub struct Program {
    pub functions: Vec<String>,
    pub module_slots: i32,
    pub software_id: i64,
    pub server_id: i64,
    pub resources: Option<super::models::OriginMachine>,
    pub name: String,
    pub version: i32,
    pub capability: Option<String>,
    pub capability_name: Option<String>,
    pub role: Option<String>,
    pub action_kind: Option<String>,
    pub origin: String,
    pub running: bool,
    pub stats: Stats,
    pub skills: Vec<Skill>,
}
#[derive(Serialize)]
pub struct Skill {
    pub action: String,
    pub attribute: String,
    pub threshold: i32,
    pub unlocked: bool,
}
#[derive(Deserialize)]
pub struct StartOperationRequest {
    pub capability: String,
    pub target_user_id: i64,
    pub software_id: Option<i64>,
}

#[derive(Deserialize)]
pub struct TuneProgramRequest {
    pub stats: Stats,
}
#[derive(Serialize)]
pub struct OperationStarted {
    pub process_id: i64,
    pub seconds: i32,
    pub capability: String,
    pub work_units: i32,
    pub attacker: Stats,
    pub defender: Stats,
    pub estimated_chance: String,
}

#[derive(Serialize)]
pub struct MachineAccess {
    pub ip: i64,
    pub identity: String,
    pub privilege: String,
    pub scope: String,
    pub permissions: Vec<String>,
    pub status: String,
    pub persistence_installed: bool,
}

#[derive(Serialize)]
pub struct WebContent {
    pub path: String,
    pub content: String,
    pub revision: i64,
}

/// Publicación ya confirmada. Conserva el recibo de proceso para clientes anteriores.
#[derive(Serialize)]
pub struct WebPublication {
    pub process_id: i64,
    pub seconds: i32,
    pub estimated_chance: String,
    #[serde(flatten)]
    pub web: WebContent,
}

#[derive(Deserialize)]
pub struct ModifyWebRequest {
    pub content: String,
}

#[derive(Deserialize)]
pub struct ForgeEvidenceRequest {
    pub apparent_origin_ip: i64,
}

#[derive(Serialize)]
pub struct TacticalActionStarted {
    pub process_id: i64,
    pub seconds: i32,
    pub estimated_chance: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AntivirusMode { Scan, Clean }

#[derive(Deserialize)]
pub struct AntivirusRequest {
    pub software_id: i64,
    pub mode: AntivirusMode,
}

#[derive(Serialize)]
pub struct AntivirusQueued {
    pub process_id: i64,
    pub seconds: i32,
}

#[derive(Serialize)]
pub struct EvidenceSummary {
    pub id: i64,
    pub process_id: i64,
    pub action: String,
    pub state: String,
    pub visible: bool,
    pub can_tamper: bool,
    pub apparent_origin_ip: Option<i64>,
    pub confidence: i32,
}

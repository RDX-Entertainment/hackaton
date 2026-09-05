use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct ProgramRow {
    pub software_id: i64,
    pub name: String,
    pub version: i32,
    pub capability: Option<String>,
    pub capability_name: Option<String>,
    pub role: Option<String>,
    pub action_kind: Option<String>,
    pub grants_login: Option<String>,
    pub origin: String,
    pub running: bool,
    pub effectiveness: i32,
    pub stealth: i32,
    pub detection: i32,
    pub removal: i32,
    pub penetration: i32,
    pub resistance: i32,
    pub hardening: i32,
    pub manipulation: i32,
    pub integrity: i32,
    pub anti_forensics: i32,
    pub forensics: i32,
    pub persistence: i32,
    pub stability: i32,
    pub cpu_need: i32,
    pub ram_need: i32,
    pub net_need: i32,
    pub weight: i32,
}

#[derive(Debug, Clone, FromRow)]
pub struct CapabilityRow {
    pub id: i16,
    pub code: String,
    pub name: String,
    pub domain: String,
    pub role: String,
    pub action_kind: String,
    pub relevant_attributes: serde_json::Value,
    pub secondary_attributes: serde_json::Value,
    pub available_actions: serde_json::Value,
    pub max_access_scope: String,
    pub granted_permissions: serde_json::Value,
}

#[derive(Debug, Clone, FromRow)]
pub struct ModuleRow {
    pub id: i64,
    pub code: String,
    pub name: String,
    pub capability: Option<String>,
    pub effectiveness_delta: i32,
    pub stealth_delta: i32,
    pub detection_delta: i32,
    pub removal_delta: i32,
    pub penetration_delta: i32,
    pub resistance_delta: i32,
    pub hardening_delta: i32,
    pub manipulation_delta: i32,
    pub integrity_delta: i32,
    pub anti_forensics_delta: i32,
    pub forensics_delta: i32,
    pub persistence_delta: i32,
    pub stability_delta: i32,
    pub cpu_delta: i32,
    pub ram_delta: i32,
    pub net_delta: i32,
    pub size_delta: i32,
}

#[derive(Debug, Clone, FromRow)]
pub struct BlueprintRow {
    pub id: i64,
    pub name: String,
    pub revision: i32,
    pub capability: String,
    pub role: String,
}

/// El equipo desde el que sale una operacion: el que guarda el programa que la hace.
///
/// Sustituye a `HardwareTotals`, que sumaba el procesador, la memoria y la salida de **toda**
/// la flota. Un programa corre donde esta guardado —lo decidio la fase 1—, asi que la potencia
/// con la que cuenta es la de ese equipo y no la de todos juntos.
#[derive(Debug, Clone, FromRow)]
pub struct OriginMachine {
    pub server_id: i64,
    pub cpu: i32,
    pub ram: i32,
    pub net: i32,
}

#[derive(Debug, Clone, FromRow)]
pub struct MachineAccessRow {
    pub ip: i64,
    pub identity: String,
    pub privilege: String,
    pub scope: String,
    pub permissions: serde_json::Value,
    pub status: String,
    pub persistence_installed: bool,
    pub target_user_id: i64,
    pub target_is_npc: bool,
}

#[derive(Debug, Clone, FromRow)]
pub struct WebContentRow {
    pub path: String,
    pub content: String,
    pub revision: i64,
}

#[derive(Debug, Clone, FromRow)]
pub struct EvidenceRow {
    pub id: i64,
    pub process_id: i64,
    pub action: String,
    pub state: String,
    pub visible: bool,
    pub can_tamper: bool,
    pub apparent_origin_ip: Option<i64>,
    pub confidence: i32,
}

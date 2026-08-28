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

#[derive(Debug, Clone, FromRow)]
pub struct HardwareTotals {
    pub cpu: i32,
    pub ram: i32,
    pub net: i32,
}

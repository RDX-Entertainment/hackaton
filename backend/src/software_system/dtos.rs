use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct Capability {
    pub code: String,
    pub name: String,
    pub domain: String,
    pub role: String,
    pub action_kind: String,
}
#[derive(Serialize)]
pub struct ResearchModule {
    pub id: i64,
    pub code: String,
    pub name: String,
    pub capability: Option<String>,
    pub deltas: Stats,
}
#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Stats {
    pub effectiveness: i32,
    pub stealth: i32,
    pub detection: i32,
    pub removal: i32,
    pub penetration: i32,
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
    pub software_id: i64,
    pub name: String,
    pub version: i32,
    pub capability: Option<String>,
    pub capability_name: Option<String>,
    pub role: Option<String>,
    pub action_kind: Option<String>,
    pub origin: String,
    pub running: bool,
    pub stats: Stats,
}
#[derive(Deserialize)]
pub struct StartOperationRequest {
    pub capability: String,
    pub target_user_id: i64,
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
}

use serde::{Deserialize, Serialize};

use super::models::MissionRow;

//
// ==============================
// BOARD
// ==============================
//

#[derive(Debug, Serialize)]
pub struct MissionResponse {
    pub id: i64,

    // `type` es palabra reservada en Rust; la columna y el JSON conservan el nombre.
    #[serde(rename = "type")]
    pub mission_type: i16,

    pub status: i16,
    pub hirer: i64,
    pub victim: i64,
    pub info: i64,
    pub info2: i64,
    pub prize: i64,
    pub level: i16,
    pub objective: &'static str,
    pub target_name: String,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub reward_module_name: Option<String>,
    pub reward_module_tier: Option<i32>,
}

impl From<MissionRow> for MissionResponse {
    fn from(value: MissionRow) -> Self {
        Self {
            id: value.id,
            mission_type: value.mission_type,
            status: value.status,
            hirer: value.hirer,
            victim: value.victim,
            info: value.info,
            info2: value.info2,
            prize: value.prize,
            level: value.level,
            objective: "gain_access",
            target_name: value.target_name,
            completed_at: value.completed_at,
            reward_module_name: value.reward_module_name,
            reward_module_tier: value.reward_module_tier,
        }
    }
}

//
// ==============================
// COMMANDS
// ==============================
//

#[derive(Debug, Deserialize)]
pub struct MissionCommandRequest {
    pub action: String,
}

#[derive(Debug, Serialize)]
pub struct MissionCommandResponse {
    pub id: i64,
    pub action: String,
}

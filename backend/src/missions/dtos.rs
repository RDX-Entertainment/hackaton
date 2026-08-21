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

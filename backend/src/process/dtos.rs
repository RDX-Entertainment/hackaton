use serde::{Deserialize, Serialize};

use super::models::ProcessRow;

//
// ==============================
// QUEUE
// ==============================
//

#[derive(Debug, Serialize)]
pub struct ProcessResponse {
    pub id: i64,
    pub victim_id: i64,
    pub action: i16,
    pub info: String,
    pub details: String,
    pub ideal_seconds: i32,
    pub worked_seconds: i32,
    pub paused: bool,
    pub completed: bool,
    pub status: String,
    pub result_code: Option<String>,
    pub result_message: Option<String>,
}

impl From<ProcessRow> for ProcessResponse {
    fn from(value: ProcessRow) -> Self {
        Self {
            id: value.pid,
            victim_id: value.victim_id,
            action: value.action,
            info: value.info,
            details: value.info_str,
            ideal_seconds: value.time_ideal,
            worked_seconds: value.time_worked,
            paused: value.is_paused,
            completed: value
                .completed_at
                .is_some(),
            status: value.status,
            result_code: value.result_code,
            result_message: value.result_message,
        }
    }
}

//
// ==============================
// START
// ==============================
//

#[derive(Debug, Deserialize)]
pub struct StartProcessRequest {
    pub action: u8,
    pub victim_id: Option<i64>,
    pub software_id: Option<i64>,
    pub info: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ProcessStartedResponse {
    pub id: i64,
    pub action: u8,
    pub seconds: i32,
}

//
// ==============================
// PAUSE / RESUME
// ==============================
//

#[derive(Debug, Serialize)]
pub struct ProcessPausedResponse {
    pub id: i64,
    pub paused: bool,
    pub seconds_left: i32,
}

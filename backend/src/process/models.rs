use chrono::{DateTime, Utc};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct ProcessRow {
    pub pid: i64,
    pub victim_id: i64,
    pub action: i16,
    pub info: String,
    pub info_str: String,
    pub time_ideal: i32,
    pub time_worked: i32,
    pub is_paused: bool,
    pub completed_at: Option<DateTime<Utc>>,
    pub status: String,
    pub result_code: Option<String>,
    pub result_message: Option<String>,
}

use chrono::{DateTime, Utc};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct SoftwareRow {
    pub id: i64,
    pub soft_name: String,
    pub soft_version: i32,
    pub soft_size: i32,
    pub soft_ram: i32,
    pub soft_type: i16,
    pub soft_hidden: bool,
    pub soft_hidden_with: i32,
    pub is_folder: bool,
    pub running: bool,
}

#[derive(Debug, Clone, FromRow)]
pub struct RemoteSoftwareRow {
    pub id: i64,
    pub soft_name: String,
    pub soft_version: i32,
    pub soft_size: i32,
    pub soft_ram: i32,
    pub soft_type: i16,
    pub is_folder: bool,
}

#[derive(Debug, Clone, FromRow)]
pub struct TransferSoftwareRow {
    pub soft_type: i16,
    pub soft_name: String,
    pub soft_size: i32,
}

#[derive(Debug, Clone, FromRow)]
pub struct ResearchHistoryRow {
    pub id: i64,
    pub software_id: i64,
    pub software_type: i16,
    pub new_version: i32,
    pub software_name: String,
    pub researched_at: DateTime<Utc>,
}
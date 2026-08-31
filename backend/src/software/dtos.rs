use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::models::{
    RemoteSoftwareRow,
    ResearchHistoryRow,
    SoftwareRow,
};

//
// ==============================
// SOFTWARE TYPES
// ==============================
//

#[derive(Debug, Serialize)]
pub struct SoftwareTypeResponse {
    pub name: String,
    pub id: i16,
}

//
// ==============================
// SOFTWARE LOCAL
// ==============================
//

#[derive(Debug, Serialize)]
pub struct SoftwareResponse {
    pub id: i64,
    pub name: String,
    pub version: i32,
    pub size: i32,
    pub ram: i32,

    #[serde(rename = "type")]
    pub soft_type: i16,

    pub hidden: bool,
    pub hidden_with: i32,
    pub running: bool,
    pub folder: bool,
    /// En que equipo vive. Sin esto la interfaz no puede agrupar por maquina, que es lo
    /// unico que hace legible una lista cuando el software deja de ser «del jugador».
    pub server_id: i64,
}

impl From<SoftwareRow> for SoftwareResponse {
    fn from(value: SoftwareRow) -> Self {
        Self {
            id: value.id,
            name: value.soft_name,
            version: value.soft_version,
            size: value.soft_size,
            ram: value.soft_ram,
            soft_type: value.soft_type,
            hidden: value.soft_hidden,
            hidden_with: value.soft_hidden_with,
            running: value.running,
            folder: value.is_folder,
            server_id: value.server_id,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct NewSoftwareRequest {
    pub name: String,
    pub soft_type: i16,
    pub size: i32,
    pub ram: i32,
    /// En que equipo se escribe. Un programa nace en un disco, no «en el jugador».
    pub server_id: i64,
}

#[derive(Debug, Serialize)]
pub struct SoftwareCreatedResponse {
    pub id: i64,
    pub name: String,
    pub version: i32,
}

//
// ==============================
// SOFTWARE REMOTO
// ==============================
//

#[derive(Debug, Serialize)]
pub struct RemoteSoftwareResponse {
    pub id: i64,
    pub name: String,
    pub version: i32,
    pub size: i32,
    pub ram: i32,

    #[serde(rename = "type")]
    pub soft_type: i16,

    pub folder: bool,
}

impl From<RemoteSoftwareRow> for RemoteSoftwareResponse {
    fn from(value: RemoteSoftwareRow) -> Self {
        Self {
            id: value.id,
            name: value.soft_name,
            version: value.soft_version,
            size: value.soft_size,
            ram: value.soft_ram,
            soft_type: value.soft_type,
            folder: value.is_folder,
        }
    }
}

//
// ==============================
// TRANSFER
// ==============================
//

#[derive(Debug, Deserialize)]
pub struct TransferRequest {
    pub direction: String,
    pub software_id: i64,
}

#[derive(Debug, Serialize)]
pub struct TransferStartedResponse {
    pub process_id: i64,
    pub action: i16,
    pub seconds: i32,
}

#[derive(Debug, Serialize)]
pub struct DeleteSoftwareStartedResponse {
    pub process_id: i64,
    pub seconds: i32,
}

//
// ==============================
// RESEARCH
// ==============================
//

#[derive(Debug, Serialize)]
pub struct ResearchHistoryResponse {
    pub id: i64,
    pub software_id: i64,

    #[serde(rename = "type")]
    pub software_type: i16,

    pub version: i32,
    pub name: String,
    pub researched_at: DateTime<Utc>,
}

impl From<ResearchHistoryRow> for ResearchHistoryResponse {
    fn from(value: ResearchHistoryRow) -> Self {
        Self {
            id: value.id,
            software_id: value.software_id,
            software_type: value.software_type,
            version: value.new_version,
            name: value.software_name,
            researched_at: value.researched_at,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ResearchStartRequest {
    pub software_id: i64,
}

#[derive(Debug, Serialize)]
pub struct ResearchStartedResponse {
    pub process_id: i64,
    pub software_id: i64,
    pub seconds: i32,
}

/// Entrada del panel: a diferencia de la creación normal, el administrador también define
/// versión y si el programa queda cargado en memoria.
#[derive(Debug, Deserialize)]
pub struct AdminSoftwareRequest {
    pub name: String,
    pub soft_type: i16,
    pub version: i32,
    pub size: i32,
    pub ram: i32,
    pub running: bool,
}

#[derive(Debug, Deserialize)]
pub struct AdminSoftwareRunningRequest {
    pub running: bool,
}

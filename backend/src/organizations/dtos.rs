use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use super::models::{
    BehaviorRow, NetworkNpcRow, NetworkRow, NodeLinkRow, NpcCatalogRow, OrganizationRow,
};

#[derive(Debug, Deserialize)]
pub struct BehaviorInput {
    pub code: String,

    /// Cada comportamiento define sus propias claves. Exigir un objeto evita guardar un
    /// escalar sin nombre que ningun modulo podria ampliar sin romper clientes.
    #[serde(default = "empty_object")]
    pub config: Value,
}

fn empty_object() -> Value {
    json!({})
}

#[derive(Debug, Deserialize)]
pub struct OrganizationInput {
    pub slug: String,
    pub name: String,
    pub category: String,

    #[serde(default)]
    pub description: String,

    #[serde(default = "active_by_default")]
    pub active: bool,

    #[serde(default)]
    pub behaviors: Vec<BehaviorInput>,
}

fn active_by_default() -> bool {
    true
}

#[derive(Debug, Serialize)]
pub struct BehaviorResponse {
    pub code: String,
    pub config: Value,
}

impl From<BehaviorRow> for BehaviorResponse {
    fn from(value: BehaviorRow) -> Self {
        Self {
            code: value.code,
            // `config` viene de JSONB; el fallback solo protege una base alterada a mano.
            config: serde_json::from_str(&value.config).unwrap_or_else(|_| json!({})),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct OrganizationResponse {
    pub id: i64,
    pub slug: String,
    pub name: String,
    pub category: String,
    pub description: String,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub behaviors: Vec<BehaviorResponse>,
    pub networks: Vec<NetworkResponse>,
    pub links: Vec<NodeLinkResponse>,
}

impl OrganizationResponse {
    pub fn new(
        row: OrganizationRow,
        behaviors: Vec<BehaviorResponse>,
        networks: Vec<NetworkResponse>,
        links: Vec<NodeLinkResponse>,
    ) -> Self {
        Self {
            id: row.id,
            slug: row.slug,
            name: row.name,
            category: row.category,
            description: row.description,
            active: row.active,
            created_at: row.created_at,
            updated_at: row.updated_at,
            behaviors,
            networks,
            links,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct NodeLinkInput {
    pub from_npc_id: i64,
    pub to_npc_id: i64,
}
#[derive(Debug, Serialize)]
pub struct NodeLinkResponse {
    pub id: i64,
    pub from_npc_id: i64,
    pub to_npc_id: i64,
}
impl From<NodeLinkRow> for NodeLinkResponse {
    fn from(value: NodeLinkRow) -> Self {
        Self {
            id: value.id,
            from_npc_id: value.from_npc_id,
            to_npc_id: value.to_npc_id,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct NetworkInput {
    pub name: String,

    #[serde(default = "internal_zone")]
    pub zone: String,
}

fn internal_zone() -> String {
    "internal".to_owned()
}

#[derive(Debug, Deserialize)]
pub struct NetworkNpcInput {
    pub npc_id: i64,

    #[serde(default = "server_role")]
    pub role: String,
}

#[derive(Debug, Deserialize)]
pub struct NodeInput {
    pub zone: String,
    pub map_x: i32,
    pub map_y: i32,

    #[serde(default = "server_role")]
    pub role: String,
}

fn server_role() -> String {
    "server".to_owned()
}

#[derive(Debug, Serialize)]
pub struct NetworkNpcResponse {
    pub id: i64,
    pub ip: i64,
    #[serde(rename = "type")]
    pub npc_type: i16,
    pub name: String,
    pub role: String,
    pub source_kit_id: Option<i64>,
    pub zone: String,
    pub map_x: i32,
    pub map_y: i32,
    pub lan_available: i64,
    pub lan_total: i64,
    pub computer_components: i64,
}

impl From<NetworkNpcRow> for NetworkNpcResponse {
    fn from(value: NetworkNpcRow) -> Self {
        Self {
            id: value.npc_id,
            ip: value.npc_ip,
            npc_type: value.npc_type,
            name: value.npc_name,
            role: value.role,
            source_kit_id: value.source_kit_id,
            zone: value.zone,
            map_x: value.map_x,
            map_y: value.map_y,
            lan_available: value.lan_available,
            lan_total: value.lan_total,
            computer_components: value.computer_components,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct NetworkResponse {
    pub id: i64,
    pub name: String,
    pub zone: String,
    pub created_at: DateTime<Utc>,
    pub npcs: Vec<NetworkNpcResponse>,
}

impl NetworkResponse {
    pub fn new(row: NetworkRow, npcs: Vec<NetworkNpcResponse>) -> Self {
        Self {
            id: row.id,
            name: row.name,
            zone: row.zone,
            created_at: row.created_at,
            npcs,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct NpcCatalogResponse {
    pub id: i64,
    pub ip: i64,
    #[serde(rename = "type")]
    pub npc_type: i16,
    pub name: String,
    pub organization_id: Option<i64>,
    pub network_id: Option<i64>,
    pub zone: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateServerInput {
    pub name: String,
    pub kit_id: i64,

    #[serde(default = "server_role")]
    pub role: String,

    #[serde(default = "neutral_npc_type")]
    pub npc_type: i16,

    #[serde(default)]
    pub web: String,
}

/// A creative build never consumes inventory.  It is intentionally separate from a kit:
/// a kit is a reusable catalogue recipe, while these lines are the final machine layout.
#[derive(Debug, Deserialize)]
pub struct ComputerBuildInput {
    #[serde(default)]
    pub kit_id: Option<i64>,
    #[serde(default)]
    pub items: Vec<ComputerBuildItemInput>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ComputerBuildItemInput {
    pub part: i64,
    pub amount: i16,
}

#[derive(Debug, Serialize)]
pub struct ComputerConfigurationResponse {
    pub source_kit_id: Option<i64>,
    pub items: Vec<ComputerBuildItemInput>,
}

fn neutral_npc_type() -> i16 {
    70
}

#[derive(Debug, Serialize)]
pub struct ServerCreatedResponse {
    pub npc_id: i64,
    pub ip: i64,
    pub password: String,
    pub machine_id: i64,
    pub kit_id: i64,
    pub components: Vec<i64>,
}

impl From<NpcCatalogRow> for NpcCatalogResponse {
    fn from(value: NpcCatalogRow) -> Self {
        Self {
            id: value.id,
            ip: value.ip,
            npc_type: value.npc_type,
            name: value.name,
            organization_id: value.organization_id,
            network_id: value.network_id,
            zone: value.zone,
        }
    }
}

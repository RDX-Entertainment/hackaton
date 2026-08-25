use chrono::{DateTime, Utc};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct OrganizationRow {
    pub id: i64,
    pub slug: String,
    pub name: String,
    pub category: String,
    pub description: String,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct BehaviorRow {
    pub organization_id: i64,
    pub code: String,

    /// Se lee como texto para que el DTO no dependa de detalles de sqlx. La columna es
    /// JSONB, asi que PostgreSQL ya garantiza que siempre sea JSON valido.
    pub config: String,
}

#[derive(Debug, Clone, FromRow)]
pub struct NetworkRow {
    pub id: i64,
    pub organization_id: i64,
    pub name: String,
    pub zone: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct NetworkNpcRow {
    pub network_id: i64,
    pub npc_id: i64,
    pub npc_ip: i64,
    pub npc_type: i16,
    pub npc_name: String,
    pub role: String,
    pub source_kit_id: Option<i64>,
    pub zone: String,
    pub map_x: i32,
    pub map_y: i32,
    pub lan_available: i64,
    pub lan_total: i64,
    pub computer_components: i64,
}

/// Catalogo para el selector del panel. Incluye tambien NPCs que ya pertenecen a otra
/// red, porque asignarlos de nuevo los mueve de forma atomica.
#[derive(Debug, Clone, FromRow)]
pub struct NpcCatalogRow {
    pub id: i64,
    pub ip: i64,
    pub npc_type: i16,
    pub name: String,
    pub organization_id: Option<i64>,
    pub network_id: Option<i64>,
    pub zone: Option<String>,
}

#[derive(Debug, Clone, FromRow)]
pub struct NodeLinkRow { pub id: i64, pub organization_id: i64, pub from_npc_id: i64, pub to_npc_id: i64 }

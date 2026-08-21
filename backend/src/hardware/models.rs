use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct MachineRow {
    pub server_id: i64,
    pub name: String,
    pub cpu: f32,
    pub hdd: f32,
    pub ram: f32,
    pub net: f32,
}

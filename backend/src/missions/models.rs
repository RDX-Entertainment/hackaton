use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct MissionRow {
    pub id: i64,
    pub mission_type: i16,
    pub status: i16,
    pub hirer: i64,
    pub victim: i64,
    pub info: i64,
    pub info2: i64,
    pub prize: i64,
    pub level: i16,
}

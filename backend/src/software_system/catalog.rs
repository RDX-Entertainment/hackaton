use sqlx::{PgPool, Row};
use crate::error::ApiError;
use super::dtos::{Capability, ResearchModule, Stats};

pub async fn capabilities(pool: &PgPool) -> Result<Vec<Capability>, ApiError> {
    let rows = sqlx::query("SELECT c.code,c.name,d.code AS domain,c.role,c.action_kind FROM software_capabilities c JOIN software_domains d ON d.id=c.domain_id ORDER BY d.id,c.id").fetch_all(pool).await?;
    Ok(rows.into_iter().map(|row| Capability { code: row.get("code"), name: row.get("name"), domain: row.get("domain"), role: row.get("role"), action_kind: row.get("action_kind") }).collect())
}

pub async fn modules(pool: &PgPool, user_id: i64) -> Result<Vec<ResearchModule>, ApiError> {
    let rows = sqlx::query("SELECT m.*,c.code AS capability FROM research_modules m LEFT JOIN software_capabilities c ON c.id=m.capability_id JOIN user_research_modules u ON u.module_id=m.id WHERE u.user_id=$1 ORDER BY m.id").bind(user_id).fetch_all(pool).await?;
    Ok(rows.into_iter().map(|row| ResearchModule { id: row.get("id"), code: row.get("code"), name: row.get("name"), capability: row.get("capability"), deltas: Stats { effectiveness: row.get("effectiveness_delta"), stealth: row.get("stealth_delta"), detection: row.get("detection_delta"), removal: row.get("removal_delta"), penetration: row.get("penetration_delta"), stability: row.get("stability_delta"), cpu: row.get("cpu_delta"), ram: row.get("ram_delta"), net: row.get("net_delta"), weight: row.get("size_delta") } }).collect())
}

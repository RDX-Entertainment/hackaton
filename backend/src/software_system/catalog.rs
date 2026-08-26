use sqlx::{PgPool, Row};
use crate::error::ApiError;
use super::{dtos::{Capability, Program, ResearchModule, Stats}, support::profile_stats};

pub async fn programs(pool: &PgPool, user_id: i64) -> Result<Vec<Program>, ApiError> {
    let rows = sqlx::query("SELECT p.*,s.soft_name,s.soft_version,c.code,c.name AS capability_name,c.role,c.action_kind,EXISTS(SELECT 1 FROM software_running r WHERE r.soft_id=s.id) AS running FROM software_profiles p JOIN software s ON s.id=p.software_id LEFT JOIN software_capabilities c ON c.id=p.capability_id WHERE s.user_id=$1 AND NOT s.is_npc ORDER BY s.id DESC").bind(user_id).fetch_all(pool).await?;
    Ok(rows.into_iter().map(|row| Program { software_id: row.get("software_id"), name: row.get("soft_name"), version: row.get("soft_version"), capability: row.get("code"), capability_name: row.get("capability_name"), role: row.get("role"), action_kind: row.get("action_kind"), origin: row.get("origin"), running: row.get("running"), stats: profile_stats(&row) }).collect())
}

pub async fn capabilities(pool: &PgPool) -> Result<Vec<Capability>, ApiError> {
    let rows = sqlx::query("SELECT c.code,c.name,d.code AS domain,c.role,c.action_kind FROM software_capabilities c JOIN software_domains d ON d.id=c.domain_id ORDER BY d.id,c.id").fetch_all(pool).await?;
    Ok(rows.into_iter().map(|row| Capability { code: row.get("code"), name: row.get("name"), domain: row.get("domain"), role: row.get("role"), action_kind: row.get("action_kind") }).collect())
}

pub async fn modules(pool: &PgPool, user_id: i64) -> Result<Vec<ResearchModule>, ApiError> {
    let rows = sqlx::query("SELECT m.*,c.code AS capability FROM research_modules m LEFT JOIN software_capabilities c ON c.id=m.capability_id JOIN user_research_modules u ON u.module_id=m.id WHERE u.user_id=$1 ORDER BY m.id").bind(user_id).fetch_all(pool).await?;
    Ok(rows.into_iter().map(|row| ResearchModule { id: row.get("id"), code: row.get("code"), name: row.get("name"), capability: row.get("capability"), deltas: Stats { effectiveness: row.get("effectiveness_delta"), stealth: row.get("stealth_delta"), detection: row.get("detection_delta"), removal: row.get("removal_delta"), penetration: row.get("penetration_delta"), stability: row.get("stability_delta"), cpu: row.get("cpu_delta"), ram: row.get("ram_delta"), net: row.get("net_delta"), weight: row.get("size_delta") } }).collect())
}

use serde_json::json;
use sqlx::{PgPool, Row};
use crate::error::ApiError;
use super::{dtos::{Blueprint, CompilationQueued, CreateBlueprintRequest}, support::{apply_module, base_stats, blueprint_modules, owned_blueprint}};

fn projected(role: &str, modules: &[sqlx::postgres::PgRow]) -> (Vec<i64>, super::dtos::Stats) { let mut stats = base_stats(role); let mut ids = Vec::new(); for module in modules { stats = apply_module(stats, module); ids.push(module.get("id")); } (ids, stats) }

pub async fn list(pool: &PgPool, user_id: i64) -> Result<Vec<Blueprint>, ApiError> {
    let rows = sqlx::query("SELECT b.id,b.name,b.revision,c.code,c.role FROM software_blueprints b JOIN software_capabilities c ON c.id=b.capability_id WHERE b.user_id=$1 ORDER BY b.created_at DESC").bind(user_id).fetch_all(pool).await?;
    let mut result = Vec::new();
    for row in rows { let id: i64 = row.get("id"); let modules = blueprint_modules(pool, id).await?; let (module_ids, stats) = projected(row.get("role"), &modules); result.push(Blueprint { id, name: row.get("name"), revision: row.get("revision"), capability: row.get("code"), modules: module_ids, projected: stats }); }
    Ok(result)
}

pub async fn create(pool: &PgPool, user_id: i64, input: CreateBlueprintRequest) -> Result<Blueprint, ApiError> {
    let name = input.name.trim(); if name.is_empty() || name.len() > 25 { return Err(ApiError::BadRequest("blueprint name must be 1-25 characters".into())); } if input.module_ids.len() > 3 { return Err(ApiError::BadRequest("a blueprint can use at most three modules".into())); }
    let capability = sqlx::query("SELECT id,role FROM software_capabilities WHERE code=$1").bind(&input.capability).fetch_optional(pool).await?.ok_or(ApiError::BadRequest("unknown capability".into()))?; let capability_id: i16 = capability.get("id"); let role: String = capability.get("role");
    for module_id in &input.module_ids { let available: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM research_modules m JOIN user_research_modules u ON u.module_id=m.id WHERE m.id=$1 AND u.user_id=$2 AND (m.capability_id IS NULL OR m.capability_id=$3))").bind(module_id).bind(user_id).bind(capability_id).fetch_one(pool).await?; if !available { return Err(ApiError::BadRequest("module is unavailable for this capability".into())); } }
    let id: i64 = sqlx::query_scalar("INSERT INTO software_blueprints(user_id,name,capability_id) VALUES($1,$2,$3) RETURNING id").bind(user_id).bind(name).bind(capability_id).fetch_one(pool).await?;
    for module_id in &input.module_ids { sqlx::query("INSERT INTO blueprint_modules(blueprint_id,module_id) VALUES($1,$2)").bind(id).bind(module_id).execute(pool).await?; }
    let modules = blueprint_modules(pool, id).await?; let (module_ids, stats) = projected(&role, &modules); Ok(Blueprint { id, name: name.to_owned(), revision: 1, capability: input.capability, modules: module_ids, projected: stats })
}

/// Compilar ocupa disco y tiempo. La copia no aparece al pulsar el boton: el worker usa el
/// payload congelado cuando vence el proceso. Solo hay una compilacion por jugador para que
/// una cola no permita reservar el mismo disco muchas veces.
pub async fn compile(pool: &PgPool, user_id: i64, blueprint_id: i64) -> Result<CompilationQueued, ApiError> {
    let (name, capability, role, revision) = owned_blueprint(pool, user_id, blueprint_id).await?; let (_, stats) = projected(&role, &blueprint_modules(pool, blueprint_id).await?);
    let active: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM processes WHERE creator_id=$1 AND action=91 AND completed_at IS NULL)").bind(user_id).fetch_one(pool).await?;
    if active { return Err(ApiError::BadRequest("COMPILATION_ACTIVE: termina la compilacion en curso antes de iniciar otra".into())); }
    let (capacity, used): (i64, i64) = sqlx::query_as("SELECT COALESCE((SELECT sum(hdd)::bigint FROM hardware WHERE user_id=$1 AND NOT is_npc),0), COALESCE((SELECT sum(soft_size)::bigint FROM software WHERE user_id=$1 AND NOT is_npc),0)").bind(user_id).fetch_one(pool).await?;
    if capacity - used < i64::from(stats.weight) { return Err(ApiError::BadRequest("DISK_FULL: no hay espacio suficiente para compilar este programa".into())); }
    let seconds = (15 + stats.weight + stats.cpu / 40 + stats.ram / 32 + stats.net * 8).clamp(20, 300);
    let payload = json!({"blueprint_id": blueprint_id, "name": name, "capability": capability, "revision": revision, "stats": stats});
    let process_id: i64 = sqlx::query_scalar("INSERT INTO processes(creator_id,victim_id,action,soft_id,info,info_str,time_end,time_ideal,status,started_at,resource_snapshot,process_payload) VALUES($1,0,91,0,'compile','compile',now()+($2::integer*interval '1 second'),$2,'running',now(),$3,$4) RETURNING pid")
        .bind(user_id).bind(seconds).bind(json!({"disk_needed": stats.weight, "disk_free": capacity - used})).bind(payload).fetch_one(pool).await?;
    Ok(CompilationQueued { process_id, blueprint_id, name, capability, seconds, stats })
}

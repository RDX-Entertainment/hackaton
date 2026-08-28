use serde_json::Value;
use sqlx::PgPool;

use super::models::{BlueprintRow, CapabilityRow, HardwareTotals, ModuleRow, ProgramRow};

pub async fn programs(pool: &PgPool, user_id: i64) -> Result<Vec<ProgramRow>, sqlx::Error> {
    sqlx::query_as("SELECT p.software_id,s.soft_name AS name,s.soft_version AS version,c.code AS capability,c.name AS capability_name,c.role,c.action_kind,NULL::text AS grants_login,p.origin,EXISTS(SELECT 1 FROM software_running r WHERE r.soft_id=s.id) AS running,p.effectiveness,p.stealth,p.detection,p.removal,p.penetration,p.stability,p.cpu_need,p.ram_need,p.net_need,p.weight FROM software_profiles p JOIN software s ON s.id=p.software_id LEFT JOIN software_capabilities c ON c.id=p.capability_id WHERE s.user_id=$1 AND NOT s.is_npc ORDER BY s.id DESC").bind(user_id).fetch_all(pool).await
}

pub async fn capabilities(pool: &PgPool) -> Result<Vec<CapabilityRow>, sqlx::Error> {
    sqlx::query_as("SELECT c.id,c.code,c.name,d.code AS domain,c.role,c.action_kind FROM software_capabilities c JOIN software_domains d ON d.id=c.domain_id ORDER BY d.id,c.id").fetch_all(pool).await
}

pub async fn modules(pool: &PgPool, user_id: i64) -> Result<Vec<ModuleRow>, sqlx::Error> {
    sqlx::query_as("SELECT m.id,m.code,m.name,c.code AS capability,m.effectiveness_delta,m.stealth_delta,m.detection_delta,m.removal_delta,m.penetration_delta,m.stability_delta,m.cpu_delta,m.ram_delta,m.net_delta,m.size_delta FROM research_modules m LEFT JOIN software_capabilities c ON c.id=m.capability_id JOIN user_research_modules u ON u.module_id=m.id WHERE u.user_id=$1 ORDER BY m.id").bind(user_id).fetch_all(pool).await
}

pub async fn tune(
    pool: &PgPool,
    user_id: i64,
    software_id: i64,
    stats: &(i32, i32, i32, i32, i32, i32, i32, i32, i32, i32),
) -> Result<bool, sqlx::Error> {
    let (effectiveness, stealth, detection, removal, penetration, stability, cpu, ram, net, weight) =
        *stats;
    Ok(sqlx::query("UPDATE software_profiles p SET effectiveness=$3,stealth=$4,detection=$5,removal=$6,penetration=$7,stability=$8,cpu_need=$9,ram_need=$10,net_need=$11,weight=$12 FROM software s WHERE p.software_id=s.id AND s.id=$1 AND s.user_id=$2 AND NOT s.is_npc")
        .bind(software_id).bind(user_id).bind(effectiveness).bind(stealth).bind(detection).bind(removal).bind(penetration).bind(stability).bind(cpu).bind(ram).bind(net).bind(weight).execute(pool).await?.rows_affected()>0)
}

pub async fn blueprints(pool: &PgPool, user_id: i64) -> Result<Vec<BlueprintRow>, sqlx::Error> {
    sqlx::query_as("SELECT b.id,b.name,b.revision,c.code AS capability,c.role FROM software_blueprints b JOIN software_capabilities c ON c.id=b.capability_id WHERE b.user_id=$1 ORDER BY b.created_at DESC").bind(user_id).fetch_all(pool).await
}
pub async fn blueprint_modules(
    pool: &PgPool,
    blueprint_id: i64,
) -> Result<Vec<ModuleRow>, sqlx::Error> {
    sqlx::query_as("SELECT m.id,m.code,m.name,c.code AS capability,m.effectiveness_delta,m.stealth_delta,m.detection_delta,m.removal_delta,m.penetration_delta,m.stability_delta,m.cpu_delta,m.ram_delta,m.net_delta,m.size_delta FROM blueprint_modules bm JOIN research_modules m ON m.id=bm.module_id LEFT JOIN software_capabilities c ON c.id=m.capability_id WHERE bm.blueprint_id=$1 ORDER BY m.id").bind(blueprint_id).fetch_all(pool).await
}
pub async fn capability(pool: &PgPool, code: &str) -> Result<Option<CapabilityRow>, sqlx::Error> {
    sqlx::query_as("SELECT c.id,c.code,c.name,d.code AS domain,c.role,c.action_kind FROM software_capabilities c JOIN software_domains d ON d.id=c.domain_id WHERE c.code=$1").bind(code).fetch_optional(pool).await
}
pub async fn module_available(
    pool: &PgPool,
    user: i64,
    module: i64,
    capability: i16,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM research_modules m JOIN user_research_modules u ON u.module_id=m.id WHERE m.id=$1 AND u.user_id=$2 AND (m.capability_id IS NULL OR m.capability_id=$3))").bind(module).bind(user).bind(capability).fetch_one(pool).await
}
pub async fn insert_blueprint(
    pool: &PgPool,
    user: i64,
    name: &str,
    capability: i16,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        "INSERT INTO software_blueprints(user_id,name,capability_id) VALUES($1,$2,$3) RETURNING id",
    )
    .bind(user)
    .bind(name)
    .bind(capability)
    .fetch_one(pool)
    .await
}
pub async fn add_blueprint_module(
    pool: &PgPool,
    blueprint: i64,
    module: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO blueprint_modules(blueprint_id,module_id) VALUES($1,$2)")
        .bind(blueprint)
        .bind(module)
        .execute(pool)
        .await?;
    Ok(())
}
pub async fn owned_blueprint(
    pool: &PgPool,
    user: i64,
    blueprint: i64,
) -> Result<Option<BlueprintRow>, sqlx::Error> {
    sqlx::query_as("SELECT b.id,b.name,b.revision,c.code AS capability,c.role FROM software_blueprints b JOIN software_capabilities c ON c.id=b.capability_id WHERE b.id=$1 AND b.user_id=$2").bind(blueprint).bind(user).fetch_optional(pool).await
}
pub async fn compilation_active(pool: &PgPool, user: i64) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM processes WHERE creator_id=$1 AND action=91 AND completed_at IS NULL)").bind(user).fetch_one(pool).await
}
pub async fn disk(pool: &PgPool, user: i64) -> Result<(i64, i64), sqlx::Error> {
    sqlx::query_as("SELECT COALESCE((SELECT sum(hdd)::bigint FROM hardware WHERE user_id=$1 AND NOT is_npc),0),COALESCE((SELECT sum(soft_size)::bigint FROM software WHERE user_id=$1 AND NOT is_npc),0)").bind(user).fetch_one(pool).await
}
pub async fn enqueue_compilation(
    pool: &PgPool,
    user: i64,
    seconds: i32,
    snapshot: Value,
    payload: Value,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar("INSERT INTO processes(creator_id,victim_id,action,soft_id,info,info_str,time_end,time_ideal,status,started_at,resource_snapshot,process_payload) VALUES($1,0,91,0,'compile','compile',now()+($2::integer*interval '1 second'),$2,'running',now(),$3,$4) RETURNING pid").bind(user).bind(seconds).bind(snapshot).bind(payload).fetch_one(pool).await
}
pub async fn attacker(
    pool: &PgPool,
    user: i64,
    capability: &str,
) -> Result<Option<ProgramRow>, sqlx::Error> {
    sqlx::query_as("SELECT p.software_id,s.soft_name AS name,s.soft_version AS version,c.code AS capability,c.name AS capability_name,c.role,c.action_kind,c.grants_login,p.origin,true AS running,p.effectiveness,p.stealth,p.detection,p.removal,p.penetration,p.stability,p.cpu_need,p.ram_need,p.net_need,p.weight FROM software_profiles p JOIN software s ON s.id=p.software_id JOIN software_running r ON r.soft_id=s.id JOIN software_capabilities c ON c.id=p.capability_id AND c.code=$2 WHERE s.user_id=$1 AND NOT s.is_npc ORDER BY p.effectiveness+p.penetration DESC LIMIT 1").bind(user).bind(capability).fetch_optional(pool).await
}
pub async fn defender(
    pool: &PgPool,
    user: i64,
    capability: &str,
) -> Result<Option<ProgramRow>, sqlx::Error> {
    sqlx::query_as("SELECT p.software_id,s.soft_name AS name,s.soft_version AS version,c.code AS capability,c.name AS capability_name,c.role,c.action_kind,NULL::text AS grants_login,p.origin,true AS running,p.effectiveness,p.stealth,p.detection,p.removal,p.penetration,p.stability,p.cpu_need,p.ram_need,p.net_need,p.weight FROM software_profiles p JOIN software s ON s.id=p.software_id JOIN software_running r ON r.soft_id=s.id JOIN software_capabilities c ON c.id=p.capability_id JOIN capability_counters x ON x.defender_capability_id=p.capability_id WHERE s.user_id=$1 AND NOT s.is_npc AND x.attacker_capability_id=(SELECT id FROM software_capabilities WHERE code=$2) ORDER BY p.effectiveness+p.detection+p.removal DESC LIMIT 1").bind(user).bind(capability).fetch_optional(pool).await
}
pub async fn hardware(pool: &PgPool, user: i64) -> Result<HardwareTotals, sqlx::Error> {
    sqlx::query_as("SELECT COALESCE(sum(cpu),0)::integer AS cpu,COALESCE(sum(ram),0)::integer AS ram,COALESCE(sum(net),0)::integer AS net FROM hardware WHERE user_id=$1 AND NOT is_npc").bind(user).fetch_one(pool).await
}
pub async fn enqueue_operation(
    pool: &PgPool,
    user: i64,
    target: i64,
    software: i64,
    capability: &str,
    seconds: i32,
    snapshot: Value,
    payload: Value,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar("INSERT INTO processes(creator_id,victim_id,action,soft_id,info,info_str,time_end,time_ideal,status,started_at,resource_snapshot,process_payload) VALUES($1,$2,90,$3,$4,$4,now()+($5::integer*interval '1 second'),$5,'running',now(),$6,$7) RETURNING pid").bind(user).bind(target).bind(software).bind(capability).bind(seconds).bind(snapshot).bind(payload).fetch_one(pool).await
}

use serde_json::json;
use sqlx::PgPool;

use crate::error::ApiError;

use super::{
    dtos::{
        Blueprint, Capability, CompilationQueued, CreateBlueprintRequest, OperationStarted,
        Program, ResearchModule, StartOperationRequest, Stats,
    },
    models::{ModuleRow, ProgramRow},
    repo,
};

fn stats(program: &ProgramRow) -> Stats {
    Stats {
        effectiveness: program.effectiveness,
        stealth: program.stealth,
        detection: program.detection,
        removal: program.removal,
        penetration: program.penetration,
        stability: program.stability,
        cpu: program.cpu_need,
        ram: program.ram_need,
        net: program.net_need,
        weight: program.weight,
    }
}
fn module_stats(module: &ModuleRow) -> Stats {
    Stats {
        effectiveness: module.effectiveness_delta,
        stealth: module.stealth_delta,
        detection: module.detection_delta,
        removal: module.removal_delta,
        penetration: module.penetration_delta,
        stability: module.stability_delta,
        cpu: module.cpu_delta,
        ram: module.ram_delta,
        net: module.net_delta,
        weight: module.size_delta,
    }
}
fn base_stats(role: &str) -> Stats {
    if role == "defender" {
        Stats {
            effectiveness: 45,
            detection: 50,
            removal: 35,
            stealth: 0,
            penetration: 0,
            stability: 65,
            cpu: 200,
            ram: 128,
            net: 1,
            weight: 24,
        }
    } else {
        Stats {
            effectiveness: 50,
            stealth: 45,
            detection: 0,
            removal: 0,
            penetration: 45,
            stability: 55,
            cpu: 200,
            ram: 128,
            net: 1,
            weight: 24,
        }
    }
}
fn apply(mut stats: Stats, module: &ModuleRow) -> Stats {
    let delta = module_stats(module);
    stats.effectiveness = (stats.effectiveness + delta.effectiveness).clamp(0, 100);
    stats.stealth = (stats.stealth + delta.stealth).clamp(0, 100);
    stats.detection = (stats.detection + delta.detection).clamp(0, 100);
    stats.removal = (stats.removal + delta.removal).clamp(0, 100);
    stats.penetration = (stats.penetration + delta.penetration).clamp(0, 100);
    stats.stability = (stats.stability + delta.stability).clamp(0, 100);
    stats.cpu = (stats.cpu + delta.cpu).max(1);
    stats.ram = (stats.ram + delta.ram).max(1);
    stats.net = (stats.net + delta.net).max(1);
    stats.weight = (stats.weight + delta.weight).max(1);
    stats
}
async fn projected(
    pool: &PgPool,
    role: &str,
    blueprint: i64,
) -> Result<(Vec<i64>, Stats), ApiError> {
    let modules = repo::blueprint_modules(pool, blueprint).await?;
    let ids = modules.iter().map(|m| m.id).collect();
    Ok((ids, modules.iter().fold(base_stats(role), apply)))
}

pub async fn programs(pool: &PgPool, user: i64) -> Result<Vec<Program>, ApiError> {
    Ok(repo::programs(pool, user)
        .await?
        .into_iter()
        .map(|p| {
            let program_stats = stats(&p);
            Program {
                software_id: p.software_id,
                name: p.name,
                version: p.version,
                capability: p.capability,
                capability_name: p.capability_name,
                role: p.role,
                action_kind: p.action_kind,
                origin: p.origin,
                running: p.running,
                stats: program_stats,
            }
        })
        .collect())
}
pub async fn capabilities(pool: &PgPool) -> Result<Vec<Capability>, ApiError> {
    Ok(repo::capabilities(pool)
        .await?
        .into_iter()
        .map(|c| Capability {
            code: c.code,
            name: c.name,
            domain: c.domain,
            role: c.role,
            action_kind: c.action_kind,
        })
        .collect())
}
pub async fn modules(pool: &PgPool, user: i64) -> Result<Vec<ResearchModule>, ApiError> {
    Ok(repo::modules(pool, user)
        .await?
        .into_iter()
        .map(|m| {
            let deltas = module_stats(&m);
            ResearchModule {
                id: m.id,
                code: m.code,
                name: m.name,
                capability: m.capability,
                deltas,
            }
        })
        .collect())
}
pub async fn tune_program(
    pool: &PgPool,
    user: i64,
    software: i64,
    stats: Stats,
) -> Result<(), ApiError> {
    let attrs = [
        stats.effectiveness,
        stats.stealth,
        stats.detection,
        stats.removal,
        stats.penetration,
        stats.stability,
    ];
    if attrs.iter().any(|v| !(0..=100).contains(v))
        || stats.cpu < 1
        || stats.ram < 1
        || stats.net < 1
        || stats.weight < 1
    {
        return Err(ApiError::BadRequest("invalid laboratory attributes".into()));
    }
    if !repo::tune(
        pool,
        user,
        software,
        &(
            stats.effectiveness,
            stats.stealth,
            stats.detection,
            stats.removal,
            stats.penetration,
            stats.stability,
            stats.cpu,
            stats.ram,
            stats.net,
            stats.weight,
        ),
    )
    .await?
    {
        return Err(ApiError::NotFound);
    }
    Ok(())
}
pub async fn blueprints(pool: &PgPool, user: i64) -> Result<Vec<Blueprint>, ApiError> {
    let mut out = Vec::new();
    for b in repo::blueprints(pool, user).await? {
        let (modules, projected) = projected(pool, &b.role, b.id).await?;
        out.push(Blueprint {
            id: b.id,
            name: b.name,
            revision: b.revision,
            capability: b.capability,
            modules,
            projected,
        });
    }
    Ok(out)
}
pub async fn create_blueprint(
    pool: &PgPool,
    user: i64,
    input: CreateBlueprintRequest,
) -> Result<Blueprint, ApiError> {
    let name = input.name.trim();
    if name.is_empty() || name.len() > 25 || input.module_ids.len() > 3 {
        return Err(ApiError::BadRequest("invalid blueprint".into()));
    }
    let capability = repo::capability(pool, &input.capability)
        .await?
        .ok_or_else(|| ApiError::BadRequest("unknown capability".into()))?;
    for module in &input.module_ids {
        if !repo::module_available(pool, user, *module, capability.id).await? {
            return Err(ApiError::BadRequest(
                "module is unavailable for this capability".into(),
            ));
        }
    }
    let id = repo::insert_blueprint(pool, user, name, capability.id).await?;
    for module in input.module_ids {
        repo::add_blueprint_module(pool, id, module).await?;
    }
    let (modules, projected) = projected(pool, &capability.role, id).await?;
    Ok(Blueprint {
        id,
        name: name.to_owned(),
        revision: 1,
        capability: capability.code,
        modules,
        projected,
    })
}
pub async fn compile(
    pool: &PgPool,
    user: i64,
    blueprint: i64,
) -> Result<CompilationQueued, ApiError> {
    let b = repo::owned_blueprint(pool, user, blueprint)
        .await?
        .ok_or(ApiError::NotFound)?;
    let (_, stats) = projected(pool, &b.role, b.id).await?;
    if repo::compilation_active(pool, user).await? {
        return Err(ApiError::BadRequest(
            "COMPILATION_ACTIVE: termina la compilacion en curso antes de iniciar otra".into(),
        ));
    }
    let (capacity, used) = repo::disk(pool, user).await?;
    if capacity - used < i64::from(stats.weight) {
        return Err(ApiError::BadRequest(
            "DISK_FULL: no hay espacio suficiente para compilar este programa".into(),
        ));
    }
    let calculated =
        (15 + stats.weight + stats.cpu / 40 + stats.ram / 32 + stats.net * 8).clamp(20, 300);
    let seconds = crate::process::services::duration_for(calculated);
    let name = b.name.clone();
    let capability = b.capability.clone();
    let payload = json!({"blueprint_id":b.id,"name":name,"capability":capability,"revision":b.revision,"stats":stats});
    let process_id = repo::enqueue_compilation(
        pool,
        user,
        seconds,
        json!({"disk_needed":stats.weight,"disk_free":capacity-used}),
        payload,
    )
    .await?;
    Ok(CompilationQueued {
        process_id,
        blueprint_id: b.id,
        name,
        capability,
        seconds,
        stats,
    })
}
pub async fn start_operation(
    pool: &PgPool,
    user: i64,
    input: StartOperationRequest,
) -> Result<OperationStarted, ApiError> {
    if input.target_user_id == user {
        return Err(ApiError::BadRequest(
            "an operation needs another player as target".into(),
        ));
    }
    let capability = input.capability;
    let attacker = repo::attacker(pool, user, &capability)
        .await?
        .ok_or_else(|| {
            ApiError::BadRequest("run a compiled program with that capability first".into())
        })?;
    let defender = repo::defender(pool, input.target_user_id, &capability).await?;
    let attacker_stats = stats(&attacker);
    let defender_stats = defender.as_ref().map(stats).unwrap_or_default();
    let hardware = repo::hardware(pool, user).await?;
    if hardware.cpu < attacker_stats.cpu
        || hardware.ram < attacker_stats.ram
        || hardware.net < attacker_stats.net
    {
        return Err(ApiError::BadRequest(
            "hardware cannot run this operation".into(),
        ));
    }
    let work_units =
        (120 + defender_stats.effectiveness + defender_stats.detection + defender_stats.removal
            - attacker_stats.penetration)
            .max(40);
    let power =
        (attacker_stats.effectiveness + attacker_stats.penetration + attacker_stats.stability)
            .max(1);
    let pressure = (attacker_stats.cpu * 100 / hardware.cpu.max(1)).max(
        (attacker_stats.ram * 100 / hardware.ram.max(1))
            .max(attacker_stats.net * 100 / hardware.net.max(1)),
    );
    let seconds = crate::process::services::duration_for(
        ((work_units * 10) / (power * 100 / (100 + pressure)).max(1)).clamp(20, 900),
    );
    let payload = json!({"capability":capability,"target_user_id":input.target_user_id,"action_kind":attacker.action_kind,"grants_login":attacker.grants_login,"attack_effectiveness":attacker_stats.effectiveness,"attack_penetration":attacker_stats.penetration,"attack_stealth":attacker_stats.stealth,"defense_effectiveness":defender_stats.effectiveness,"defense_detection":defender_stats.detection,"defense_removal":defender_stats.removal});
    let process_id=repo::enqueue_operation(pool,user,input.target_user_id,attacker.software_id,&capability,seconds,json!({"cpu":attacker_stats.cpu,"ram":attacker_stats.ram,"net":attacker_stats.net,"attacker":attacker_stats,"defender":defender_stats}),payload).await?;
    Ok(OperationStarted {
        process_id,
        seconds,
        capability,
        work_units,
        attacker: attacker_stats,
        defender: defender_stats,
    })
}

use rand::Rng;
use serde_json::json;
use sqlx::PgPool;

use crate::error::ApiError;

use super::{
    combat::{self, Resolution, RollSource},
    dtos::{
        Blueprint, Capability, CompilationQueued, CreateBlueprintRequest, EvidenceSummary,
        ForgeEvidenceRequest, MachineAccess, ModifyWebRequest, OperationStarted, Program,
        ResearchModule, StartOperationRequest, Stats, TacticalActionStarted, WebContent,
    },
    models::{ModuleRow, ProgramRow},
    repo,
};

struct RuntimeRolls;

impl RollSource for RuntimeRolls {
    fn roll_percent(&mut self) -> u8 {
        rand::rng().random_range(1..=100)
    }
}

fn json_strings(value: &serde_json::Value) -> Vec<String> {
    value
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|entry| entry.as_str().map(str::to_owned))
        .collect()
}

fn stats(program: &ProgramRow) -> Stats {
    Stats {
        effectiveness: program.effectiveness,
        resistance: program.resistance,
        stealth: program.stealth,
        detection: program.detection,
        removal: program.removal,
        penetration: program.penetration,
        hardening: program.hardening,
        manipulation: program.manipulation,
        integrity: program.integrity,
        anti_forensics: program.anti_forensics,
        forensics: program.forensics,
        persistence: program.persistence,
        stability: program.stability,
        cpu: program.cpu_need,
        ram: program.ram_need,
        net: program.net_need,
        weight: program.weight,
    }
}

/// Una capacidad declara sus atributos relevantes en datos. Esta puntuación vive aquí para
/// que cada acción elija una sola copia activa por su función, sin sumar copias idénticas.
fn suitability(program: &ProgramRow, attributes: &[String]) -> i32 {
    let values = stats(program);
    attributes
        .iter()
        .map(|attribute| match attribute.as_str() {
            "effectiveness" => values.effectiveness,
            "resistance" => values.resistance,
            "penetration" => values.penetration,
            "hardening" => values.hardening,
            "stealth" => values.stealth,
            "detection" => values.detection,
            "manipulation" => values.manipulation,
            "integrity" => values.integrity,
            "anti_forensics" => values.anti_forensics,
            "forensics" => values.forensics,
            "persistence" => values.persistence,
            "removal" => values.removal,
            "stability" => values.stability,
            _ => 0,
        })
        .sum()
}
fn select_best(candidates: Vec<ProgramRow>, attributes: &[String]) -> Option<ProgramRow> {
    candidates
        .into_iter()
        .max_by_key(|program| (suitability(program, attributes), program.software_id))
}
async fn selected_attacker(
    pool: &PgPool,
    user: i64,
    capability: &str,
) -> Result<ProgramRow, ApiError> {
    let definition = repo::capability(pool, capability)
        .await?
        .ok_or_else(|| ApiError::BadRequest("unknown capability".into()))?;
    let attributes = json_strings(&definition.relevant_attributes);
    select_best(
        repo::attacker_candidates(pool, user, capability).await?,
        &attributes,
    )
    .ok_or_else(|| ApiError::BadRequest("REQUIRED_SOFTWARE_NOT_RUNNING".into()))
}
async fn selected_defender(
    pool: &PgPool,
    user: i64,
    capability: &str,
) -> Result<Option<ProgramRow>, ApiError> {
    let candidates = repo::defender_candidates(pool, user, capability).await?;
    // La defensa también se elige por la declaración de su propia capacidad. Por
    // ejemplo, un IDS prioriza detection/forensics y un monitor de integridad
    // integrity/forensics; no existe un orden universal heredado.
    let mut selected: Option<(i32, ProgramRow)> = None;
    for candidate in candidates {
        let attributes = match candidate.capability.as_deref() {
            Some(code) => repo::capability(pool, code)
                .await?
                .map(|definition| json_strings(&definition.relevant_attributes))
                .unwrap_or_default(),
            None => Vec::new(),
        };
        let score = suitability(&candidate, &attributes);
        if selected
            .as_ref()
            .map(|(best, _)| score > *best)
            .unwrap_or(true)
        {
            selected = Some((score, candidate));
        }
    }
    Ok(selected.map(|(_, program)| program))
}

fn requires_unlocked_skill(capability: &str, stats: &Stats, action: &str) -> bool {
    super::skills::unlocked(Some(capability), stats)
        .iter()
        .any(|(candidate, _, _, unlocked)| *candidate == action && *unlocked)
}

/// Plan de fases puro. El trabajador recibe estas dos resoluciones ya congeladas y no
/// consulta perfiles ni vuelve a lanzar dados al aplicar una acción terminada.
fn tactical_phase_plan(
    action: i16,
    attack: &Stats,
    defense: &Stats,
    rolls: &mut impl RollSource,
) -> (Resolution, Resolution) {
    let primary = match action {
        92 => combat::resolve(attack.manipulation, defense.integrity, rolls),
        93 | 94 => combat::resolve(attack.anti_forensics, defense.forensics, rolls),
        _ => combat::resolve(0, 100, rolls),
    };
    let detection = combat::resolve(defense.detection, attack.stealth, rolls);
    (primary, detection)
}

fn intrusion_phase_plan(
    attack: &Stats,
    defense: &Stats,
    rolls: &mut impl RollSource,
) -> (Resolution, Resolution, Resolution) {
    (
        combat::resolve(attack.effectiveness, defense.resistance, rolls),
        combat::resolve(attack.penetration, defense.hardening, rolls),
        combat::resolve(defense.detection, attack.stealth, rolls),
    )
}
fn module_stats(module: &ModuleRow) -> Stats {
    Stats {
        effectiveness: module.effectiveness_delta,
        resistance: module.resistance_delta,
        stealth: module.stealth_delta,
        detection: module.detection_delta,
        removal: module.removal_delta,
        penetration: module.penetration_delta,
        hardening: module.hardening_delta,
        manipulation: module.manipulation_delta,
        integrity: module.integrity_delta,
        anti_forensics: module.anti_forensics_delta,
        forensics: module.forensics_delta,
        persistence: module.persistence_delta,
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
            resistance: 55,
            detection: 50,
            removal: 35,
            stealth: 0,
            penetration: 0,
            hardening: 50,
            manipulation: 0,
            integrity: 45,
            anti_forensics: 0,
            forensics: 45,
            persistence: 0,
            stability: 65,
            cpu: 200,
            ram: 128,
            net: 1,
            weight: 24,
        }
    } else {
        Stats {
            effectiveness: 50,
            resistance: 0,
            stealth: 45,
            detection: 0,
            removal: 0,
            penetration: 45,
            hardening: 0,
            manipulation: 25,
            integrity: 0,
            anti_forensics: 20,
            forensics: 0,
            persistence: 20,
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
    stats.resistance = (stats.resistance + delta.resistance).clamp(0, 100);
    stats.hardening = (stats.hardening + delta.hardening).clamp(0, 100);
    stats.manipulation = (stats.manipulation + delta.manipulation).clamp(0, 100);
    stats.integrity = (stats.integrity + delta.integrity).clamp(0, 100);
    stats.anti_forensics = (stats.anti_forensics + delta.anti_forensics).clamp(0, 100);
    stats.forensics = (stats.forensics + delta.forensics).clamp(0, 100);
    stats.persistence = (stats.persistence + delta.persistence).clamp(0, 100);
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
            let skills = super::skills::unlocked(p.capability.as_deref(), &program_stats)
                .into_iter()
                .map(
                    |(action, attribute, threshold, unlocked)| super::dtos::Skill {
                        action: action.to_owned(),
                        attribute: attribute.to_owned(),
                        threshold,
                        unlocked,
                    },
                )
                .collect();
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
                skills,
            }
        })
        .collect())
}

pub async fn accesses(pool: &PgPool, user: i64) -> Result<Vec<MachineAccess>, ApiError> {
    Ok(repo::accesses(pool, user)
        .await?
        .into_iter()
        .map(|a| MachineAccess {
            ip: a.ip,
            identity: a.identity,
            privilege: a.privilege,
            scope: a.scope,
            permissions: json_strings(&a.permissions),
            status: a.status,
            persistence_installed: a.persistence_installed,
        })
        .collect())
}
pub async fn web_content(pool: &PgPool, user: i64, ip: i64) -> Result<WebContent, ApiError> {
    let access = repo::access(pool, user, ip)
        .await?
        .ok_or_else(|| ApiError::BadRequest("NO_ACCESS".into()))?;
    if !json_strings(&access.permissions)
        .iter()
        .any(|p| p == "read_files")
    {
        return Err(ApiError::BadRequest("INSUFFICIENT_PERMISSION".into()));
    }
    let content = repo::web_content(pool, access.target_user_id, access.target_is_npc).await?;
    Ok(WebContent {
        path: content.path,
        content: content.content,
        revision: content.revision,
    })
}
pub async fn evidence(pool: &PgPool, user: i64) -> Result<Vec<EvidenceSummary>, ApiError> {
    Ok(repo::evidence(pool, user)
        .await?
        .into_iter()
        .map(|e| EvidenceSummary {
            id: e.id,
            process_id: e.process_id,
            action: e.action,
            state: e.state,
            visible: e.visible,
            can_tamper: e.can_tamper,
            apparent_origin_ip: e.apparent_origin_ip,
            confidence: e.confidence,
        })
        .collect())
}

fn has_permission(access: &super::models::MachineAccessRow, permission: &str) -> bool {
    json_strings(&access.permissions)
        .iter()
        .any(|item| item == permission)
}

pub async fn modify_web(
    pool: &PgPool,
    user: i64,
    ip: i64,
    input: ModifyWebRequest,
) -> Result<TacticalActionStarted, ApiError> {
    if input.content.trim().is_empty() || input.content.len() > 20_000 {
        return Err(ApiError::BadRequest("INVALID_WEB_CONTENT".into()));
    }
    start_tactical(
        pool,
        user,
        ip,
        "web_manipulator",
        "write_files",
        92,
        json!({"content":input.content,"path":"/index.html"}),
    )
    .await
}

pub async fn hide_evidence(
    pool: &PgPool,
    user: i64,
    ip: i64,
    evidence: i64,
) -> Result<TacticalActionStarted, ApiError> {
    let target = repo::evidence_for_action(pool, user, evidence)
        .await?
        .ok_or(ApiError::NotFound)?;
    let access = repo::access(pool, user, ip)
        .await?
        .ok_or_else(|| ApiError::BadRequest("NO_ACCESS".into()))?;
    if access.target_user_id != target.0 || access.target_is_npc != target.1 {
        return Err(ApiError::BadRequest("NO_ACCESS".into()));
    }
    start_tactical(
        pool,
        user,
        ip,
        "forensic_scrubber",
        "modify_logs",
        93,
        json!({"evidence_id":evidence}),
    )
    .await
}

pub async fn forge_evidence(
    pool: &PgPool,
    user: i64,
    ip: i64,
    evidence: i64,
    input: ForgeEvidenceRequest,
) -> Result<TacticalActionStarted, ApiError> {
    if !repo::known_access(pool, user, input.apparent_origin_ip).await? {
        return Err(ApiError::BadRequest("UNKNOWN_FORGERY_REFERENCE".into()));
    }
    let target = repo::evidence_for_action(pool, user, evidence)
        .await?
        .ok_or(ApiError::NotFound)?;
    let access = repo::access(pool, user, ip)
        .await?
        .ok_or_else(|| ApiError::BadRequest("NO_ACCESS".into()))?;
    if access.target_user_id != target.0 || access.target_is_npc != target.1 {
        return Err(ApiError::BadRequest("NO_ACCESS".into()));
    }
    start_tactical(
        pool,
        user,
        ip,
        "forensic_scrubber",
        "forge_logs",
        94,
        json!({"evidence_id":evidence,"apparent_origin_ip":input.apparent_origin_ip}),
    )
    .await
}

pub async fn analyze_evidence(
    pool: &PgPool,
    user: i64,
    evidence: i64,
) -> Result<TacticalActionStarted, ApiError> {
    if !repo::evidence_for_target(pool, user, evidence).await? {
        return Err(ApiError::NotFound);
    }
    let analyzer = selected_attacker(pool, user, "forensic_analyzer").await?;
    let stats = stats(&analyzer);
    if !requires_unlocked_skill("forensic_analyzer", &stats, "analyze_evidence") {
        return Err(ApiError::BadRequest("SKILL_LOCKED".into()));
    }
    let hardware = repo::hardware(pool, user).await?;
    if hardware.cpu < stats.cpu || hardware.ram < stats.ram || hardware.net < stats.net {
        return Err(ApiError::BadRequest("INSUFFICIENT_RESOURCES".into()));
    }
    let mut rolls = RuntimeRolls;
    let resolution = combat::resolve(stats.forensics, 40, &mut rolls);
    let seconds = crate::process::services::duration_for(
        (30 + stats.cpu / 40 + (100 - stats.stability) / 2).clamp(20, 120),
    );
    let process=repo::enqueue_tactical(pool,user,user,analyzer.software_id,95,seconds,json!({"cpu":stats.cpu,"ram":stats.ram,"net":stats.net}),json!({"capability":"forensic_analyzer","phase_plan":{"primary":{"probability":resolution.probability,"roll":resolution.roll},"detection":{"probability":100,"roll":1}},"payload":{"evidence_id":evidence}})).await?;
    Ok(TacticalActionStarted {
        process_id: process,
        seconds,
        estimated_chance: chance_label(resolution.probability).to_owned(),
    })
}

async fn start_tactical(
    pool: &PgPool,
    user: i64,
    ip: i64,
    capability: &str,
    permission: &str,
    action: i16,
    extra: serde_json::Value,
) -> Result<TacticalActionStarted, ApiError> {
    let access = repo::access(pool, user, ip)
        .await?
        .ok_or_else(|| ApiError::BadRequest("NO_ACCESS".into()))?;
    if !has_permission(&access, permission) {
        return Err(ApiError::BadRequest("INSUFFICIENT_PERMISSION".into()));
    }
    let attacker = selected_attacker(pool, user, capability).await?;
    let defender = selected_defender(pool, access.target_user_id, capability).await?;
    let attack = stats(&attacker);
    let skill = match action {
        92 => "modify_web",
        93 => "hide_evidence",
        94 => "forge_evidence",
        _ => return Err(ApiError::BadRequest("UNKNOWN_TACTICAL_ACTION".into())),
    };
    if !requires_unlocked_skill(capability, &attack, skill) {
        return Err(ApiError::BadRequest("SKILL_LOCKED".into()));
    }
    let defense = defender.as_ref().map(stats).unwrap_or_default();
    let hardware = repo::hardware(pool, user).await?;
    if hardware.cpu < attack.cpu || hardware.ram < attack.ram || hardware.net < attack.net {
        return Err(ApiError::BadRequest("INSUFFICIENT_RESOURCES".into()));
    }
    let mut rolls = RuntimeRolls;
    let (primary, detection) = tactical_phase_plan(action, &attack, &defense, &mut rolls);
    let seconds = crate::process::services::duration_for(
        (40 + attack.cpu / 40 + attack.ram / 32 + (100 - attack.stability) / 2).clamp(20, 180),
    );
    let process=repo::enqueue_tactical(pool,user,access.target_user_id,attacker.software_id,action,seconds,json!({"cpu":attack.cpu,"ram":attack.ram,"net":attack.net}),json!({"ip":ip,"target_is_npc":access.target_is_npc,"capability":capability,"permission":permission,"frozen":{"attacker":attack,"defender":defense,"access":{"identity":access.identity,"privilege":access.privilege,"scope":access.scope,"permissions":access.permissions}},"phase_plan":{"primary":{"probability":primary.probability,"roll":primary.roll},"detection":{"probability":detection.probability,"roll":detection.roll}},"payload":extra})).await?;
    Ok(TacticalActionStarted {
        process_id: process,
        seconds,
        estimated_chance: chance_label(primary.probability).to_owned(),
    })
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
            relevant_attributes: json_strings(&c.relevant_attributes),
            secondary_attributes: json_strings(&c.secondary_attributes),
            available_actions: json_strings(&c.available_actions),
            max_access_scope: c.max_access_scope,
            granted_permissions: json_strings(&c.granted_permissions),
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
        stats.resistance,
        stats.stealth,
        stats.detection,
        stats.removal,
        stats.penetration,
        stats.hardening,
        stats.manipulation,
        stats.integrity,
        stats.anti_forensics,
        stats.forensics,
        stats.persistence,
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
    if !repo::tune(pool, user, software, &stats).await? {
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
    let attacker = selected_attacker(pool, user, &capability)
        .await
        .map_err(|_| {
            ApiError::BadRequest("run a compiled program with that capability first".into())
        })?;
    let defender = selected_defender(pool, input.target_user_id, &capability).await?;
    let attacker_stats = stats(&attacker);
    let defender_stats = defender.as_ref().map(stats).unwrap_or_default();
    let capability_definition = repo::capability(pool, &capability)
        .await?
        .ok_or_else(|| ApiError::BadRequest("unknown capability".into()))?;
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
    // Las tiradas se generan una vez, al encolar, y se guardan junto a las estadísticas.
    // El worker solo aplica este plan: reiniciarlo no puede repetir ni cambiar la suerte.
    let mut rolls = RuntimeRolls;
    let (execution, penetration, detection) =
        intrusion_phase_plan(&attacker_stats, &defender_stats, &mut rolls);
    let estimated_chance =
        chance_label((execution.probability + penetration.probability) / 2).to_owned();
    let payload = json!({"capability":capability,"target_user_id":input.target_user_id,"action_kind":attacker.action_kind,"grants_login":attacker.grants_login,"access_scope":capability_definition.max_access_scope,"granted_permissions":capability_definition.granted_permissions,"attack_effectiveness":attacker_stats.effectiveness,"attack_penetration":attacker_stats.penetration,"attack_stealth":attacker_stats.stealth,"defense_resistance":defender_stats.resistance,"defense_hardening":defender_stats.hardening,"defense_detection":defender_stats.detection,"phase_plan":{"execution":{"probability":execution.probability,"roll":execution.roll},"penetration":{"probability":penetration.probability,"roll":penetration.roll},"detection":{"probability":detection.probability,"roll":detection.roll}}});
    let process_id=repo::enqueue_operation(pool,user,input.target_user_id,attacker.software_id,&capability,seconds,json!({"cpu":attacker_stats.cpu,"ram":attacker_stats.ram,"net":attacker_stats.net,"attacker":attacker_stats,"defender":defender_stats}),payload).await?;
    Ok(OperationStarted {
        process_id,
        seconds,
        capability,
        work_units,
        attacker: attacker_stats,
        defender: defender_stats,
        estimated_chance,
    })
}

fn chance_label(probability: i32) -> &'static str {
    match probability {
        ..=20 => "muy baja",
        ..=40 => "baja",
        ..=60 => "equilibrada",
        ..=80 => "alta",
        _ => "muy alta",
    }
}

/// La víctima no necesita conocer un id oculto: el análisis busca una discontinuidad en
/// su propia máquina y el worker revela solo aquello que consiga reconstruir.
pub async fn discover_evidence(pool: &PgPool, user: i64) -> Result<TacticalActionStarted, ApiError> {
    let evidence = repo::hidden_evidence_for_target(pool, user)
        .await?
        .ok_or_else(|| ApiError::BadRequest("NO_HIDDEN_EVIDENCE".into()))?;
    analyze_evidence(pool, user, evidence).await
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Rolls(Vec<u8>);
    impl RollSource for Rolls {
        fn roll_percent(&mut self) -> u8 {
            self.0.remove(0)
        }
    }

    #[test]
    fn penetration_and_detection_are_independent() {
        let attack = Stats { effectiveness: 50, penetration: 50, stealth: 0, ..Default::default() };
        let defense = Stats { resistance: 50, hardening: 50, detection: 50, ..Default::default() };
        // Las dos primeras fases ganan al 50%; detección también, pero con un orden de
        // tirada propio. El último 1 prueba que entrar no implica ocultarse.
        let mut rolls = Rolls(vec![1, 1, 1]);
        let (execution, penetration, detection) = intrusion_phase_plan(&attack, &defense, &mut rolls);
        assert!(execution.success && penetration.success && detection.success);
    }

    #[test]
    fn web_modification_uses_manipulation_not_level_or_penetration() {
        let attack = Stats { manipulation: 25, stealth: 100, penetration: 100, ..Default::default() };
        let defense = Stats { integrity: 25, detection: 0, ..Default::default() };
        let mut rolls = Rolls(vec![51, 100]);
        let (modification, detection) = tactical_phase_plan(92, &attack, &defense, &mut rolls);
        assert!(!modification.success);
        assert!(!detection.success);
    }
}

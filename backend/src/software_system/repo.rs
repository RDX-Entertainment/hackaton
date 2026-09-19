use serde_json::Value;
use sqlx::PgPool;

use super::dtos::Stats;
use super::models::{
    BlueprintRow, CapabilityRow, EvidenceRow, MachineAccessRow, ModuleRow, OriginMachine,
    ProgramRow, WebContentRow,
};

pub async fn programs(pool: &PgPool, user_id: i64) -> Result<Vec<ProgramRow>, sqlx::Error> {
    sqlx::query_as("SELECT p.software_id,s.server_id,s.soft_name AS name,s.soft_version AS version,c.code AS capability,c.name AS capability_name,c.role,c.action_kind,NULL::text AS grants_login,p.origin,EXISTS(SELECT 1 FROM software_running r WHERE r.soft_id=s.id) AS running,p.effectiveness,p.stealth,p.detection,p.removal,p.penetration,p.resistance,p.hardening,p.manipulation,p.integrity,p.anti_forensics,p.forensics,p.persistence,p.stability,p.cpu_need,p.ram_need,p.net_need,p.weight FROM effective_software_profiles p JOIN software s ON s.id=p.software_id LEFT JOIN software_capabilities c ON c.id=p.capability_id WHERE s.user_id=$1 AND NOT s.is_npc AND NOT s.soft_hidden ORDER BY s.id DESC").bind(user_id).fetch_all(pool).await
}

pub async fn capabilities(pool: &PgPool) -> Result<Vec<CapabilityRow>, sqlx::Error> {
    sqlx::query_as("SELECT c.id,c.code,c.name,d.code AS domain,c.role,c.action_kind,c.relevant_attributes,c.secondary_attributes,c.available_actions,c.max_access_scope,c.granted_permissions FROM software_capabilities c JOIN software_domains d ON d.id=c.domain_id ORDER BY d.id,c.id").fetch_all(pool).await
}

pub async fn gateway_server(pool: &PgPool, user: i64) -> Result<Option<i64>, sqlx::Error> {
    sqlx::query_scalar("SELECT server_id FROM hardware WHERE user_id=$1 AND NOT is_npc AND is_gateway AND public_ip IS NOT NULL")
        .bind(user).fetch_optional(pool).await
}

pub async fn addressed_server(pool: &PgPool, user: i64, npc: bool, ip: i64) -> Result<Option<i64>, sqlx::Error> {
    sqlx::query_scalar("SELECT server_id FROM hardware WHERE user_id=$1 AND is_npc=$2 AND (public_ip=$3 OR lan_ip=$3)")
        .bind(user).bind(npc).bind(ip).fetch_optional(pool).await
}

pub async fn modules(pool: &PgPool, user_id: i64) -> Result<Vec<ModuleRow>, sqlx::Error> {
    sqlx::query_as("SELECT m.id,m.code,m.name,c.code AS capability,m.effectiveness_delta,m.stealth_delta,m.detection_delta,m.removal_delta,m.penetration_delta,m.resistance_delta,m.hardening_delta,m.manipulation_delta,m.integrity_delta,m.anti_forensics_delta,m.forensics_delta,m.persistence_delta,m.stability_delta,m.cpu_delta,m.ram_delta,m.net_delta,m.size_delta FROM research_modules m LEFT JOIN software_capabilities c ON c.id=m.capability_id JOIN user_research_modules u ON u.module_id=m.id WHERE u.user_id=$1 ORDER BY m.id").bind(user_id).fetch_all(pool).await
}

pub async fn tune(
    pool: &PgPool,
    user_id: i64,
    software_id: i64,
    stats: &Stats,
) -> Result<bool, sqlx::Error> {
    Ok(sqlx::query("UPDATE software_profiles p SET effectiveness=$3,resistance=$4,stealth=$5,detection=$6,removal=$7,penetration=$8,hardening=$9,manipulation=$10,integrity=$11,anti_forensics=$12,forensics=$13,persistence=$14,stability=$15,cpu_need=$16,ram_need=$17,net_need=$18,weight=$19 FROM software s WHERE p.software_id=s.id AND s.id=$1 AND s.user_id=$2 AND NOT s.is_npc AND NOT s.soft_hidden")
        .bind(software_id).bind(user_id).bind(stats.effectiveness).bind(stats.resistance).bind(stats.stealth).bind(stats.detection).bind(stats.removal).bind(stats.penetration).bind(stats.hardening).bind(stats.manipulation).bind(stats.integrity).bind(stats.anti_forensics).bind(stats.forensics).bind(stats.persistence).bind(stats.stability).bind(stats.cpu).bind(stats.ram).bind(stats.net).bind(stats.weight).execute(pool).await?.rows_affected()>0)
}

pub async fn blueprints(pool: &PgPool, user_id: i64) -> Result<Vec<BlueprintRow>, sqlx::Error> {
    sqlx::query_as("SELECT b.id,b.name,b.revision,c.code AS capability,c.role FROM software_blueprints b JOIN software_capabilities c ON c.id=b.capability_id WHERE b.user_id=$1 ORDER BY b.created_at DESC").bind(user_id).fetch_all(pool).await
}
pub async fn blueprint_modules(
    pool: &PgPool,
    blueprint_id: i64,
) -> Result<Vec<ModuleRow>, sqlx::Error> {
    sqlx::query_as("SELECT m.id,m.code,m.name,c.code AS capability,m.effectiveness_delta,m.stealth_delta,m.detection_delta,m.removal_delta,m.penetration_delta,m.resistance_delta,m.hardening_delta,m.manipulation_delta,m.integrity_delta,m.anti_forensics_delta,m.forensics_delta,m.persistence_delta,m.stability_delta,m.cpu_delta,m.ram_delta,m.net_delta,m.size_delta FROM blueprint_modules bm JOIN research_modules m ON m.id=bm.module_id LEFT JOIN software_capabilities c ON c.id=m.capability_id WHERE bm.blueprint_id=$1 ORDER BY m.id").bind(blueprint_id).fetch_all(pool).await
}
pub async fn capability(pool: &PgPool, code: &str) -> Result<Option<CapabilityRow>, sqlx::Error> {
    let code = match code { "antivirus_detect" | "antivirus_remove" => "antivirus", other => other };
    sqlx::query_as("SELECT c.id,c.code,c.name,d.code AS domain,c.role,c.action_kind,c.relevant_attributes,c.secondary_attributes,c.available_actions,c.max_access_scope,c.granted_permissions FROM software_capabilities c JOIN software_domains d ON d.id=c.domain_id WHERE c.code=$1").bind(code).fetch_optional(pool).await
}
pub async fn insert_blueprint(
    pool: &PgPool,
    user: i64,
    name: &str,
    capability: i16,
    modules: &[i64],
) -> Result<i64, sqlx::Error> {
    let mut tx = pool.begin().await?;
    let id = sqlx::query_scalar(
        "INSERT INTO software_blueprints(user_id,name,capability_id) VALUES($1,$2,$3) RETURNING id",
    )
    .bind(user)
    .bind(name)
    .bind(capability)
    .fetch_one(&mut *tx)
    .await?;
    for module in modules {
        sqlx::query("INSERT INTO blueprint_modules(blueprint_id,module_id) VALUES($1,$2)")
            .bind(id).bind(module).execute(&mut *tx).await?;
    }
    tx.commit().await?;
    Ok(id)
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
/// El hueco mas grande que tiene el jugador **en un solo disco**.
///
/// Antes esto sumaba todos sus equipos, que era la cuenta de antes de la 0047: entonces el
/// software era del dueno y el disco tambien. Ahora un programa vive en una maquina, asi que
/// preguntar por la suma respondia que si a compilaciones que luego no cabian en ningun sitio
/// —y el jugador se enteraba cinco minutos despues, al terminar el proceso—. La pregunta que
/// importa es en cual cabe, y esta es la respuesta a la misma que se hace el worker al
/// aterrizar el fichero.
pub async fn largest_free_disk(pool: &PgPool, user: i64) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        SELECT COALESCE(max(h.hdd::bigint - o.used), 0)
          FROM hardware h
          LEFT JOIN LATERAL (
              SELECT COALESCE(sum(s.soft_size)::bigint, 0) AS used
                FROM software s
               WHERE s.server_id = h.server_id
          ) o ON TRUE
         WHERE h.user_id = $1 AND NOT h.is_npc AND h.operational AND h.node_kind='computer'
        "#,
    )
    .bind(user)
    .fetch_one(pool)
    .await
}
/// La compilacion es el unico proceso que **no** nace en una maquina, y a proposito.
///
/// Cuando se encola todavia no hay programa —es lo que se esta haciendo— y en que equipo
/// aterriza el resultado lo decide el worker al terminar. Ponerle un origen aqui obligaria a
/// elegir maquina antes de tiempo, y esa eleccion es una regla de juego que no esta tomada.
/// `origin_server` se queda en `NULL`, que significa «no se sabe».
pub async fn enqueue_compilation(
    pool: &PgPool,
    user: i64,
    seconds: i32,
    snapshot: Value,
    payload: Value,
) -> Result<Option<i64>, sqlx::Error> {
    let mut tx = pool.begin().await?;
    // Dos peticiones del mismo jugador se serializan antes de comprobar la cola.
    sqlx::query("SELECT id FROM users WHERE id=$1 FOR UPDATE").bind(user).fetch_one(&mut *tx).await?;
    let active: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM processes WHERE creator_id=$1 AND action=91 AND completed_at IS NULL)")
        .bind(user).fetch_one(&mut *tx).await?;
    if active { return Ok(None); }
    let id = sqlx::query_scalar("INSERT INTO processes(creator_id,victim_id,action,soft_id,info,info_str,time_end,time_ideal,status,started_at,resource_snapshot,process_payload) VALUES($1,0,91,0,'compile','compile',now()+($2::integer*interval '1 second'),$2,'running',now(),$3,$4) RETURNING pid").bind(user).bind(seconds).bind(snapshot).bind(payload).fetch_one(&mut *tx).await?;
    tx.commit().await?;
    Ok(Some(id))
}
pub async fn attacker_candidates(
    pool: &PgPool,
    user: i64,
    capability: &str,
) -> Result<Vec<ProgramRow>, sqlx::Error> {
    sqlx::query_as("SELECT p.software_id,s.server_id,s.soft_name AS name,s.soft_version AS version,c.code AS capability,c.name AS capability_name,c.role,c.action_kind,c.grants_login,p.origin,true AS running,p.effectiveness,p.stealth,p.detection,p.removal,p.penetration,p.resistance,p.hardening,p.manipulation,p.integrity,p.anti_forensics,p.forensics,p.persistence,p.stability,p.cpu_need,p.ram_need,p.net_need,p.weight FROM effective_software_profiles p JOIN software s ON s.id=p.software_id JOIN software_running r ON r.soft_id=s.id JOIN hardware live ON live.server_id=s.server_id AND live.operational AND live.node_kind='computer' JOIN software_capabilities c ON c.code=$2 AND program_has_function(s.id,c.code) WHERE s.user_id=$1 AND NOT s.is_npc").bind(user).bind(capability).fetch_all(pool).await
}
pub async fn defender_candidates(
    pool: &PgPool,
    user: i64,
    capability: &str,
    npc: bool,
    server: i64,
) -> Result<Vec<ProgramRow>, sqlx::Error> {
    sqlx::query_as("SELECT p.software_id,s.server_id,s.soft_name AS name,s.soft_version AS version,c.code AS capability,c.name AS capability_name,c.role,c.action_kind,NULL::text AS grants_login,p.origin,true AS running,p.effectiveness,p.stealth,p.detection,p.removal,p.penetration,p.resistance,p.hardening,p.manipulation,p.integrity,p.anti_forensics,p.forensics,p.persistence,p.stability,p.cpu_need,p.ram_need,p.net_need,p.weight FROM effective_software_profiles p JOIN software s ON s.id=p.software_id JOIN software_running r ON r.soft_id=s.id JOIN hardware live ON live.server_id=s.server_id AND live.operational AND live.node_kind='computer' JOIN software_capabilities c ON c.code=ANY(program_functions(s.id)) JOIN capability_counters x ON x.defender_capability_id=c.id WHERE s.user_id=$1 AND s.is_npc=$3 AND s.server_id IN (SELECT h.server_id FROM hardware h JOIN hardware target ON target.server_id=$4 WHERE h.server_id=target.server_id OR (target.cluster_id IS NOT NULL AND h.cluster_id=target.cluster_id)) AND x.attacker_capability_id=(SELECT id FROM software_capabilities WHERE code=$2)").bind(user).bind(capability).bind(npc).bind(server).fetch_all(pool).await
}
/// El equipo del que sale una operacion, con lo que ese equipo puede darle.
///
/// Antes esto sumaba `WHERE user_id`: el procesador, la memoria y la salida de la flota
/// entera. Con varias maquinas por cuenta eso convertia el hardware en un deposito comun, y
/// una operacion que no cabia en ninguna maquina salia igual porque entre todas juntaban de
/// sobra. Es el mismo fallo que la memoria arrastraba hasta el 2026-08-31, y se arregla
/// igual: se mira **donde vive el programa**, que es donde corre y donde gasta.
///
/// Devuelve `None` si el programa no esta en ninguna maquina, que no deberia pasar desde la
/// fase 1 pero es mejor contestarlo que suponerlo.
pub async fn origin_machine(
    pool: &PgPool,
    software: i64,
) -> Result<Option<OriginMachine>, sqlx::Error> {
    sqlx::query_as(
        r#"
        SELECT
               -- **Si el equipo esta en un grupo, cuenta el grupo entero.** Es lo unico que
               -- hace un cluster y por lo que va el ultimo de su rumbo: suma recursos, y para
               -- sumarlos habia que haberlos separado antes. Hacia fuera el grupo se comporta
               -- como un solo ordenador, y esta es la cuenta que lo dice.
               COALESCE(g.cpu, h.cpu, 0)::integer AS cpu,
               COALESCE(g.ram, h.ram, 0)::integer AS ram,
               COALESCE(g.net, h.net, 0)::integer AS net

        FROM software s
        JOIN hardware h
            ON h.server_id = s.server_id

        LEFT JOIN LATERAL (
            SELECT sum(m.cpu) AS cpu, sum(m.ram) AS ram, sum(m.net) AS net
            FROM hardware m
            WHERE m.cluster_id = h.cluster_id AND m.operational AND m.node_kind='computer'
        ) g ON h.cluster_id IS NOT NULL

        WHERE s.id = $1 AND h.operational AND h.node_kind='computer'
        "#,
    )
    .bind(software)
    .fetch_optional(pool)
    .await
}
/// Encola una operacion contra un jugador.
///
/// **No nombra una maquina, y por eso va a su puerta principal.** Una operacion se pide contra
/// un dueno —`target_user_id` y nada mas—, sin pasar por una direccion, asi que el equipo al
/// que llega es el que lleva la del jugador. Es lo que pasaba antes sin escribirlo: `victim_ip`
/// resolvia su `game_ip`, que desde la fase 3 de direcciones es la de esa maquina. Anotarlo al
/// encolar y no deducirlo al terminar importa, porque entre una cosa y otra el jugador puede
/// haber cambiado cual manda.
///
/// La victima es siempre un jugador aqui: el worker lee `info` para decidirlo y en una
/// operacion `info` es la capacidad, nunca `'npc'`.
pub async fn enqueue_operation(
    pool: &PgPool,
    user: i64,
    target: i64,
    software: i64,
    capability: &str,
    seconds: i32,
    snapshot: Value,
    payload: Value,
    target_server: i64,
    action: i16,
) -> Result<Option<i64>, sqlx::Error> {
    let mut tx = pool.begin().await?;
    sqlx::query("SELECT id FROM users WHERE id=$1 FOR UPDATE").bind(user).fetch_one(&mut *tx).await?;
    let active: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM processes WHERE creator_id=$1 AND action=$2 AND victim_server=$3 AND completed_at IS NULL)")
        .bind(user).bind(action).bind(target_server).fetch_one(&mut *tx).await?;
    if active { return Ok(None); }
    let id = sqlx::query_scalar("INSERT INTO processes(creator_id,victim_id,action,soft_id,info,info_str,time_end,time_ideal,status,started_at,resource_snapshot,process_payload,origin_server,victim_server) VALUES($1,$2,$9,$3,$4,$4,now()+($5::integer*interval '1 second'),$5,'running',now(),$6,$7,(SELECT server_id FROM software WHERE id=$3),$8) RETURNING pid").bind(user).bind(target).bind(software).bind(capability).bind(seconds).bind(snapshot).bind(payload).bind(target_server).bind(action).fetch_one(&mut *tx).await?;
    tx.commit().await?;
    Ok(Some(id))
}

pub async fn ensure_legacy_profiles(pool: &PgPool, user: i64, npc: bool) -> Result<(), sqlx::Error> {
    sqlx::query("SELECT ensure_legacy_combat_profiles($1,$2)").bind(user).bind(npc).execute(pool).await?;
    Ok(())
}

pub async fn accesses(pool: &PgPool, user: i64) -> Result<Vec<MachineAccessRow>, sqlx::Error> {
    sqlx::query_as("SELECT target_ip AS ip,identity_name AS identity,privilege,scope,permissions,status,persistence_installed,target_user_id,target_is_npc FROM machine_accesses WHERE attacker_id=$1 ORDER BY obtained_at DESC").bind(user).fetch_all(pool).await
}
pub async fn access(
    pool: &PgPool,
    user: i64,
    ip: i64,
) -> Result<Option<MachineAccessRow>, sqlx::Error> {
    sqlx::query_as("SELECT target_ip AS ip,identity_name AS identity,privilege,scope,permissions,status,persistence_installed,target_user_id,target_is_npc FROM machine_accesses WHERE attacker_id=$1 AND target_ip=$2 AND status='active' AND (expires_at IS NULL OR expires_at>now())").bind(user).bind(ip).fetch_optional(pool).await
}
/// La pagina editable de una maquina, si tiene alguna escrita.
///
/// Devuelve `None` en vez de reventar cuando no hay ninguna. Esto no se notaba porque el
/// endpoint que lo usa era **inalcanzable**: pedia un acceso de `machine_accesses`, que solo
/// llenaba la operacion modular. En cuanto un crackeo normal empezo a conceder acceso, la
/// primera maquina sin pagina escrita contesto un 500.
///
/// Una maquina sin nada escrito no es un error: es una maquina sin nada escrito.
pub async fn web_content(
    pool: &PgPool,
    target: i64,
    npc: bool,
) -> Result<Option<WebContentRow>, sqlx::Error> {
    sqlx::query_as("SELECT path,content,COALESCE((SELECT max(id) FROM machine_web_revisions r WHERE r.target_user_id=$1 AND r.target_is_npc=$2 AND r.path=w.path),0) AS revision FROM machine_web_content w WHERE target_user_id=$1 AND target_is_npc=$2 AND path='/index.html'").bind(target).bind(npc).fetch_optional(pool).await
}
pub async fn evidence(pool: &PgPool, user: i64) -> Result<Vec<EvidenceRow>, sqlx::Error> {
    sqlx::query_as("SELECT id,process_id,action_code AS action,state,((target_user_id=$1 AND NOT target_is_npc) OR real_identity=$2) AS visible,(real_identity=$2) AS can_tamper,apparent_origin_ip,attribution_confidence AS confidence FROM game_evidence WHERE real_identity=$2 OR (target_user_id=$1 AND NOT target_is_npc AND visible_to_target) ORDER BY real_at DESC").bind(user).bind(format!("user:{user}")).fetch_all(pool).await
}

pub async fn hidden_evidence_for_target(pool: &PgPool, user: i64) -> Result<Option<i64>, sqlx::Error> {
    sqlx::query_scalar("SELECT id FROM game_evidence WHERE target_user_id=$1 AND NOT target_is_npc AND NOT visible_to_target ORDER BY real_at DESC LIMIT 1")
        .bind(user).fetch_optional(pool).await
}

/// La maquina que hay en una direccion publica.
///
/// Una accion tactica se lanza contra una direccion —es lo que el jugador tiene delante, la
/// del acceso que ya consiguio— y desde la fase 2 esa direccion **es** una maquina concreta.
/// Traducirla aqui es lo que deja escrito en el proceso a cual va, para que el efecto no
/// termine aterrizando en la principal de su dueno.
pub async fn machine_at(pool: &PgPool, ip: i64) -> Result<Option<i64>, sqlx::Error> {
    sqlx::query_scalar("SELECT server_id FROM hardware WHERE public_ip = $1")
        .bind(ip)
        .fetch_optional(pool)
        .await
}

pub async fn enqueue_tactical(
    pool: &PgPool,
    user: i64,
    target: i64,
    target_server: Option<i64>,
    software: i64,
    action: i16,
    seconds: i32,
    snapshot: Value,
    payload: Value,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar("INSERT INTO processes(creator_id,victim_id,action,soft_id,info,info_str,time_end,time_ideal,status,started_at,resource_snapshot,process_payload,origin_server,victim_server) VALUES($1,$2,$3,$4,'tactical','tactical',now()+($5::integer*interval '1 second'),$5,'running',now(),$6,$7,(SELECT server_id FROM software WHERE id=$4),$8) RETURNING pid").bind(user).bind(target).bind(action).bind(software).bind(seconds).bind(snapshot).bind(payload).bind(target_server).fetch_one(pool).await
}
pub async fn known_access(pool: &PgPool, user: i64, ip: i64) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM machine_accesses WHERE attacker_id=$1 AND target_ip=$2)",
    )
    .bind(user)
    .bind(ip)
    .fetch_one(pool)
    .await
}
pub async fn evidence_for_action(
    pool: &PgPool,
    user: i64,
    evidence: i64,
) -> Result<Option<(i64, bool)>, sqlx::Error> {
    sqlx::query_as(
        "SELECT target_user_id,target_is_npc FROM game_evidence WHERE id=$1 AND real_identity=$2",
    )
    .bind(evidence)
    .bind(format!("user:{user}"))
    .fetch_optional(pool)
    .await
}
pub async fn evidence_for_target(
    pool: &PgPool,
    user: i64,
    evidence: i64,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM game_evidence WHERE id=$1 AND target_user_id=$2 AND NOT target_is_npc)").bind(evidence).bind(user).fetch_one(pool).await
}

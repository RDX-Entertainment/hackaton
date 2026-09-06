use dotenvy::dotenv;
use sqlx::{Row, postgres::PgPoolOptions};
use std::{env, time::Duration};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    // El worker no imprimia nada. Ahora que aguanta el fallo de un proceso en vez de morirse
    // con el, poder leer cual fallo y por que deja de ser un lujo.
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    let database_url = env::var("DATABASE_URL")?;
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    let once = env::args().any(|arg| arg == "--once");
    // Los procesos se miden en segundos. Dormir un minuto entero hacia que una tarea de 20 s
    // pareciera congelada y podia terminar casi un minuto tarde. Se puede ampliar fuera de
    // desarrollo, pero el valor normal mantiene la cola perceptiblemente viva.
    let tick_seconds = env::var("WORKER_TICK_SECONDS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(5);
    loop {
        // Con `--once` se usa para probar y el fallo tiene que verse; en marcha, una vuelta
        // que falla es una vuelta, no el final del worker. Una base que parpadea, un bloqueo
        // o un dato inesperado no pueden dejar sin avanzar todo lo que tarda en el juego.
        if let Err(fallo) = tick(&pool).await {
            if once {
                return Err(fallo.into());
            }
            tracing::error!(error = %fallo, "la vuelta del worker no se pudo terminar");
        }
        if once {
            break;
        }
        tokio::time::sleep(Duration::from_secs(tick_seconds)).await;
    }
    Ok(())
}

async fn tick(pool: &sqlx::PgPool) -> Result<(), sqlx::Error> {
    resolve_completed(pool).await?;
    // Port of cron/generateMissions.php: replenish the public board from NPCs.
    for (level, desired) in [(1_i16, 50_i64), (2, 30), (3, 25)] {
        sqlx::query("WITH available AS (SELECT COUNT(*) AS total FROM missions WHERE status=1 AND level=$1), candidates AS (SELECT h.npc_ip AS hirer,v.npc_ip AS victim FROM npc h JOIN npc v ON h.id<>v.id WHERE h.npc_type=70+$1 AND v.npc_type=70+$1 AND h.down_until<=now() AND v.down_until<=now() LIMIT (SELECT GREATEST(0,$2-total) FROM available)) INSERT INTO missions(type,status,hirer,victim,prize,level) SELECT 1,1,hirer,victim,(150*$1)::bigint,$1 FROM candidates")
            .bind(level).bind(desired).execute(pool).await?;
    }
    // Port of cron/restoreSoftware.php: put back any original software an NPC has lost.
    // Without this a machine stripped by a DELETE stays stripped for good, and the world
    // wears down to nothing worth breaking into.
    // La copia vuelve a la maquina del NPC, que es donde estaba. El JOIN es tambien la
    // condicion: un NPC sin equipo no tiene disco donde reponerle nada.
    sqlx::query("INSERT INTO software(user_id,soft_name,soft_version,soft_size,soft_ram,soft_type,is_npc,server_id) SELECT o.npc_id,o.soft_name,o.soft_version,o.soft_size,o.soft_ram,o.soft_type,TRUE,h.server_id FROM software_original o JOIN hardware h ON h.user_id=o.npc_id AND h.is_npc AND h.is_gateway WHERE NOT EXISTS(SELECT 1 FROM software s WHERE s.user_id=o.npc_id AND s.is_npc AND s.soft_name=o.soft_name AND s.soft_version=o.soft_version)")
        .execute(pool).await?;
    // Port of cron/updatePremium.php.
    sqlx::query("UPDATE users SET premium=false WHERE id IN (SELECT user_id FROM users_premium WHERE premium_until < now())")
        .execute(pool).await?;
    sqlx::query("DELETE FROM users_premium WHERE premium_until < now()")
        .execute(pool)
        .await?;
    // Port of cron2/updateRanking.py, ranking users by their accumulated experience.
    sqlx::query("INSERT INTO ranking_user(user_id,rank) SELECT uid,ROW_NUMBER() OVER(ORDER BY exp DESC,uid)::integer FROM users_stats ON CONFLICT(user_id) DO UPDATE SET rank=EXCLUDED.rank")
        .execute(pool).await?;
    // Port of cron/updateServerStats.php; JSON keeps this snapshot extensible.
    sqlx::query("INSERT INTO system_stats(key,payload,updated_at) VALUES('server',jsonb_build_object('users',(SELECT count(*) FROM users),'active_users',(SELECT count(*) FROM users WHERE last_login >= now()-interval '14 days'),'clans',(SELECT count(*) FROM clans),'clan_members',(SELECT count(*) FROM clan_members),'mails',(SELECT count(*) FROM mails WHERE sender_id>0),'missions',(SELECT count(*) FROM missions),'research',(SELECT count(*) FROM research_history),'processes',(SELECT count(*) FROM processes),'software',(SELECT count(*) FROM software),'servers',(SELECT count(*) FROM hardware),'money',COALESCE((SELECT sum(cash) FROM bank_accounts),0)),now()) ON CONFLICT(key) DO UPDATE SET payload=EXCLUDED.payload,updated_at=EXCLUDED.updated_at")
        .execute(pool).await?;
    Ok(())
}

/// Port of Process::completeProcess, which the legacy source itself calls the most
/// important function of the game: the one that applies a finished process's side effects.
///
/// Until now this was a bare completion marker, so a finished HACK expired without handing
/// the player anything. Only the two actions this port can currently start are resolved;
/// every other action still just gets marked done, exactly as before.
async fn resolve_completed(pool: &sqlx::PgPool) -> Result<(), sqlx::Error> {
    let due = sqlx::query("SELECT pid,creator_id,victim_id,victim_server,action,soft_id,info FROM processes WHERE completed_at IS NULL AND NOT is_paused AND time_end<=now()")
        .fetch_all(pool).await?;
    for p in due {
        let pid: i64 = p.get("pid");
        // **Un proceso que revienta no se lleva por delante a los demas.** Antes el error
        // subia hasta `main` y el worker terminaba; al arrancar de nuevo volvia a coger el
        // mismo proceso y volvia a morir, y con la cola parada dejaba de funcionar todo lo
        // que tarda en el juego. Ahora el que falla se marca fallado —con su codigo, para
        // que se pueda mirar despues— y el siguiente sigue su camino.
        if let Err(fallo) = resolve_one(pool, &p).await {
            tracing::error!(pid, error = %fallo, "el proceso no se pudo resolver");
            // Se cierra igualmente: un proceso que no se puede resolver y se queda abierto se
            // reintenta en cada vuelta, y eso es el bucle de caida otra vez, en bonito.
            sqlx::query("UPDATE processes SET time_worked=time_ideal,completed_at=now(),status='failed',result_code='WORKER_ERROR',result_message=$2 WHERE pid=$1")
                .bind(pid)
                .bind(fallo.to_string())
                .execute(pool)
                .await?;
        }
    }
    Ok(())
}

/// Un solo proceso, de principio a fin. Separado del bucle para que su fallo sea suyo.
async fn resolve_one(pool: &sqlx::PgPool, p: &sqlx::postgres::PgRow) -> Result<(), sqlx::Error> {
    let pid: i64 = p.get("pid");
    let user: i64 = p.get("creator_id");
    let victim: i64 = p.get("victim_id");
    let action: i16 = p.get("action");
    let soft: i64 = p.get("soft_id");
    // A que maquina iba. `victim_id` dice de quien es; esto, a cual de las suyas.
    let victim_server: Option<i64> = p.get("victim_server");
    // Internet::hack writes "npc" or "player" into info when it starts the process,
    // which is what the legacy carried in its isNPC flag.
    let is_npc = p.get::<String, _>("info") == "npc";
    let exploit_succeeded = if action == 16 {
        Some(exploit_effect(pool, user, victim, is_npc, victim_server).await?)
    } else {
        None
    };
    match action {
        1 | 2 => transfer_effect(pool, user, victim, is_npc, victim_server, soft, action).await?,
        3 => delete_effect(pool, user, victim, is_npc, victim_server, soft).await?,
        4 => hide_effect(pool, user, victim, is_npc, soft).await?,
        5 => seek_effect(pool, user, victim, is_npc, soft).await?,
        13 => install_effect(pool, user, victim, is_npc, soft).await?,
        14 => uninstall_effect(pool, user, victim, is_npc, soft).await?,
        8 => edit_log_effect(pool, pid, victim, is_npc).await?,
        11 => hack_effect(pool, user, victim, is_npc, victim_server).await?,
        15 => scan_effect(pool, user, victim, is_npc, victim_server).await?,
        16 => {}
        17 => research_effect(pool, user, soft).await?,
        18 => park_effect(pool, user, soft).await?,
        19 => unpark_effect(pool, user, soft).await?,
        20 => discard_effect(pool, user, soft).await?,
        23 => analyze_effect(pool, user, victim, is_npc, victim_server, soft).await?,
        25 => reset_ip_effect(pool, user).await?,
        26 => reset_password_effect(pool, user).await?,
        90 => modular_operation_effect(pool, pid, user, victim, victim_server, soft).await?,
        91 => compilation_effect(pool, pid, user).await?,
        92 | 93 | 94 | 95 => tactical_effect(pool, pid, user, victim, action).await?,
        _ => {}
    }
    if exploit_succeeded == Some(false) {
        sqlx::query("UPDATE processes SET status='failed',result_code='EXPLOIT_BLOCKED',result_message='El firewall rechazó tus exploits instalados' WHERE pid=$1")
            .bind(pid)
            .execute(pool)
            .await?;
    }
    sqlx::query("UPDATE processes SET time_worked=time_ideal,completed_at=now(),status=CASE WHEN action IN (16,90,91,92,93,94,95) THEN status ELSE 'succeeded' END WHERE pid=$1")
        .bind(pid)
        .execute(pool)
        .await?;
    Ok(())
}

/// Un casi-acierto contra integridad no deja contenido persistente, pero sí permite
/// representar una tentativa que el monitor restauró antes de que fuera estable.
fn modification_is_partial(success: bool, probability: i32, roll: i32) -> bool {
    !success && roll <= (probability + 10).min(100)
}

/// Acciones posteriores al acceso. El payload contiene la tirada y la configuración ya
/// congeladas; el worker no vuelve a consultar estadísticas ni vuelve a sortear.
async fn tactical_effect(
    pool: &sqlx::PgPool,
    pid: i64,
    user: i64,
    victim: i64,
    action: i16,
) -> Result<(), sqlx::Error> {
    let payload: serde_json::Value =
        sqlx::query_scalar("SELECT process_payload FROM processes WHERE pid=$1")
            .bind(pid)
            .fetch_one(pool)
            .await?;
    let phase = |name: &str| {
        let v = &payload["phase_plan"][name];
        let p = v["probability"].as_i64().unwrap_or(50) as i32;
        let r = v["roll"].as_i64().unwrap_or(100) as i32;
        (p, r, r <= p)
    };
    let (prob, roll, success) = phase("primary");
    let (_, _, detected) = phase("detection");
    // Un fallo cercano conserva el intento como resultado parcial: la defensa restauró
    // el cambio antes de que quedara aplicado, pero la operación deja evidencia.
    let partial = action == 92 && modification_is_partial(success, prob, roll);
    let ip = payload["ip"].as_i64().unwrap_or(0);
    let details = &payload["payload"];
    let (code, message) = match action {
        92 => {
            if !success {
                if partial {
                    (
                        "MODIFICATION_PARTIAL",
                        "la integridad restauró una modificación parcial",
                    )
                } else if detected {
                    (
                        "MODIFICATION_BLOCKED_DETECTED",
                        "la integridad bloqueó la modificación y generó alerta",
                    )
                } else {
                    (
                        "MODIFICATION_BLOCKED",
                        "la integridad bloqueó la modificación",
                    )
                }
            } else {
                let content = details["content"].as_str().unwrap_or_default();
                let path = details["path"].as_str().unwrap_or("/index.html");
                let exists:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM game_evidence WHERE process_id=$1 AND action_code='modify_web')").bind(pid).fetch_one(pool).await?;
                if !exists {
                    let previous:Option<String>=sqlx::query_scalar("SELECT content FROM machine_web_content WHERE target_user_id=$1 AND target_is_npc=false AND path=$2").bind(victim).bind(path).fetch_optional(pool).await?;
                    if let Some(previous) = previous {
                        sqlx::query("INSERT INTO machine_web_revisions(target_user_id,target_is_npc,path,content,created_by) VALUES($1,false,$2,$3,$4)").bind(victim).bind(path).bind(previous).bind(user).execute(pool).await?;
                    }
                    sqlx::query("INSERT INTO machine_web_content(target_user_id,target_is_npc,path,content,updated_at) VALUES($1,false,$2,$3,now()) ON CONFLICT(target_user_id,target_is_npc,path) DO UPDATE SET content=EXCLUDED.content,updated_at=now()").bind(victim).bind(path).bind(content).execute(pool).await?;
                }
                if detected {
                    (
                        "MODIFIED_DETECTED",
                        "contenido web modificado; el monitor lo detectó",
                    )
                } else {
                    (
                        "MODIFIED_STEALTH",
                        "contenido web modificado sin alerta inmediata",
                    )
                }
            }
        }
        93 => {
            if success {
                ("EVIDENCE_HIDDEN", "evidencia ocultada de forma recuperable")
            } else if detected {
                (
                    "FORENSIC_BLOCKED_DETECTED",
                    "el análisis forense detectó el intento de ocultación",
                )
            } else {
                ("FORENSIC_BLOCKED", "la evidencia resistió la ocultación")
            }
        }
        95 => {
            if success {
                (
                    "EVIDENCE_RECOVERED",
                    "el análisis forense reconstruyó evidencia ocultada o alterada",
                )
            } else {
                (
                    "FORENSIC_ANALYSIS_INCONCLUSIVE",
                    "el análisis no pudo confirmar una alteración",
                )
            }
        }
        _ => {
            if success {
                ("EVIDENCE_FORGED", "procedencia aparente falsificada")
            } else if detected {
                (
                    "FORGERY_DISCOVERED",
                    "el análisis forense descubrió la falsificación",
                )
            } else {
                ("FORGERY_BLOCKED", "la falsificación no alteró la evidencia")
            }
        }
    };
    sqlx::query("INSERT INTO operation_phase_results(process_id,phase,probability,roll,outcome,details) VALUES($1,'primary',$2,$3,$4,$5) ON CONFLICT(process_id,phase) DO NOTHING")
        .bind(pid).bind(prob).bind(roll).bind(if success { "succeeded" } else if partial { "partial" } else { "blocked" })
        .bind(serde_json::json!({"partial": partial})).execute(pool).await?;
    let (detection_probability, detection_roll, detection_success) = phase("detection");
    sqlx::query("INSERT INTO operation_phase_results(process_id,phase,probability,roll,outcome,details) VALUES($1,'detection',$2,$3,$4,'{}') ON CONFLICT(process_id,phase) DO NOTHING")
        .bind(pid).bind(detection_probability).bind(detection_roll).bind(if detection_success { "detected" } else { "hidden" }).execute(pool).await?;
    if action == 92 {
        sqlx::query("INSERT INTO game_evidence(process_id,target_user_id,target_is_npc,action_code,real_origin_ip,real_identity,program_signature,concealment,state,visible_to_target,details) VALUES($1,$2,false,'modify_web',(SELECT game_ip FROM users WHERE id=$3),$4,$5,$6,$7,$8,$9) ON CONFLICT(process_id,action_code) DO NOTHING").bind(pid).bind(victim).bind(user).bind(format!("user:{user}")).bind(payload["capability"].as_str().unwrap_or("web_manipulator")).bind(if detected{0}else{70}).bind(if detected{"normal"}else{"hidden"}).bind(detected).bind(serde_json::json!({"result":code,"ip":ip})).execute(pool).await?;
    }
    if action == 93 && success {
        sqlx::query("UPDATE game_evidence SET state='degraded',concealment=GREATEST(concealment,70),visible_to_target=false WHERE id=$1 AND real_identity=$2").bind(details["evidence_id"].as_i64().unwrap_or(0)).bind(format!("user:{user}")).execute(pool).await?;
    }
    if action == 94 && success {
        sqlx::query("UPDATE game_evidence SET state='altered',apparent_origin_ip=$2,attribution_confidence=40 WHERE id=$1 AND real_identity=$3").bind(details["evidence_id"].as_i64().unwrap_or(0)).bind(details["apparent_origin_ip"].as_i64()).bind(format!("user:{user}")).execute(pool).await?;
    }
    if action == 95 && success {
        sqlx::query("UPDATE game_evidence SET state='recovered',visible_to_target=true,concealment=0,attribution_confidence=GREATEST(attribution_confidence,80) WHERE id=$1 AND target_user_id=$2")
            .bind(details["evidence_id"].as_i64().unwrap_or(0)).bind(user).execute(pool).await?;
    }
    sqlx::query("UPDATE processes SET status=$2,result_code=$3,result_message=$4 WHERE pid=$1")
        .bind(pid)
        .bind(if success { "succeeded" } else { "failed" })
        .bind(code)
        .bind(message)
        .execute(pool)
        .await?;
    Ok(())
}

/// En que disco cae un fichero que llega, y si cabe en alguno.
///
/// Desde la 0047 el software vive en una maquina y no en un dueno. Eso cambio la pregunta:
/// «cabe?» dejo de ser una cuenta sobre la suma de los equipos de alguien y paso a ser una
/// sobre un disco concreto. Los sitios que copian ficheros —compilar, transferir, infectar,
/// desaparcar— se quedaron preguntando a la vieja usanza y, peor, sin decir en que maquina
/// caia lo que insertaban. Contra una columna `NOT NULL` eso no es un fallo de cuentas: es un
/// insert que revienta, y el worker se moria con el.
///
/// La regla es el equipo con mas sitio libre, y a igualdad el principal. Es indulgente a
/// proposito —quien tiene varias maquinas no se queda sin compilar porque la principal este
/// llena— y determinista, que es lo que permite probarla.
async fn landing_disk(
    pool: &sqlx::PgPool,
    owner: i64,
    is_npc: bool,
    size: i64,
) -> Result<Option<i64>, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        SELECT h.server_id
          FROM hardware h
          LEFT JOIN LATERAL (
              SELECT COALESCE(sum(s.soft_size)::bigint, 0) AS used
                FROM software s
               WHERE s.server_id = h.server_id
          ) o ON TRUE
         WHERE h.user_id = $1
           AND h.is_npc = $2
           AND h.hdd::bigint - o.used >= $3
         ORDER BY h.hdd::bigint - o.used DESC, h.is_gateway DESC, h.server_id
         LIMIT 1
        "#,
    )
    .bind(owner)
    .bind(is_npc)
    .bind(size)
    .fetch_optional(pool)
    .await
}

/// Lo mismo, pero cuando la maquina no se elige: ya la eligio el jugador al atacarla.
///
/// Un virus se planta en el equipo en el que entraste, no en el que mejor le venga al virus.
/// Si ese esta lleno, la infeccion falla y lo dice; buscarle otro sitio seria plantarlo en una
/// maquina que nadie ha abierto.
async fn fits_on(pool: &sqlx::PgPool, server: i64, size: i64) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        SELECT h.hdd::bigint
               - COALESCE((SELECT sum(s.soft_size)::bigint FROM software s WHERE s.server_id = h.server_id), 0)
               >= $2
          FROM hardware h
         WHERE h.server_id = $1
        "#,
    )
    .bind(server)
    .bind(size)
    .fetch_optional(pool)
    .await
    .map(|found| found.unwrap_or(false))
}

/// Materializa una compilacion que ya consumio su tiempo. El espacio se comprueba de nuevo:
/// otra accion pudo haber terminado antes y ocupar el disco que estaba libre al encolarla.
async fn compilation_effect(pool: &sqlx::PgPool, pid: i64, user: i64) -> Result<(), sqlx::Error> {
    let row = sqlx::query("SELECT process_payload FROM processes WHERE pid=$1")
        .bind(pid)
        .fetch_one(pool)
        .await?;
    let payload: serde_json::Value = row.get("process_payload");
    let stats = &payload["stats"];
    let weight = stats["weight"].as_i64().unwrap_or(1).max(1);
    // `enqueue_compilation` dejo escrito que en que equipo aterriza el resultado lo decide el
    // worker al terminar. Se decide aqui, y hasta ahora no lo decidia nadie.
    let Some(server) = landing_disk(pool, user, false, weight).await? else {
        sqlx::query("UPDATE processes SET status='failed',result_code='DISK_FULL',result_message='La compilacion termino sin espacio libre en disco' WHERE pid=$1").bind(pid).execute(pool).await?;
        return Ok(());
    };
    let capability = payload["capability"].as_str().unwrap_or_default();
    let name = payload["name"].as_str().unwrap_or("Programa");
    let revision = payload["revision"].as_i64().unwrap_or(1) as i32;
    let blueprint_id = payload["blueprint_id"].as_i64();
    let legacy_type = match capability {
        "ssh_exploit" => 14,
        "ftp_exploit" => 13,
        "cracker" => 1,
        "virus_infect" => 99,
        _ => 4,
    };
    let mut tx = pool.begin().await?;
    let software_id: i64 = sqlx::query_scalar("INSERT INTO software(user_id,soft_name,soft_version,soft_size,soft_ram,soft_type,server_id) VALUES($1,$2,$3,$4,$5,$6,$7) RETURNING id")
        .bind(user).bind(name).bind(revision).bind(weight as i32).bind(stats["ram"].as_i64().unwrap_or(1).max(1) as i32).bind(legacy_type).bind(server).fetch_one(&mut *tx).await?;
    sqlx::query("INSERT INTO software_profiles(software_id,blueprint_id,origin,capability_id,effectiveness,resistance,stealth,detection,removal,penetration,hardening,manipulation,integrity,anti_forensics,forensics,persistence,stability,cpu_need,ram_need,net_need,weight) SELECT $1,$2,'research',id,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19 FROM software_capabilities WHERE code=$20")
        .bind(software_id).bind(blueprint_id)
        .bind(stats["effectiveness"].as_i64().unwrap_or(0) as i32).bind(stats["resistance"].as_i64().unwrap_or(0) as i32)
        .bind(stats["stealth"].as_i64().unwrap_or(0) as i32).bind(stats["detection"].as_i64().unwrap_or(0) as i32)
        .bind(stats["removal"].as_i64().unwrap_or(0) as i32).bind(stats["penetration"].as_i64().unwrap_or(0) as i32)
        .bind(stats["hardening"].as_i64().unwrap_or(0) as i32).bind(stats["manipulation"].as_i64().unwrap_or(0) as i32)
        .bind(stats["integrity"].as_i64().unwrap_or(0) as i32).bind(stats["anti_forensics"].as_i64().unwrap_or(0) as i32)
        .bind(stats["forensics"].as_i64().unwrap_or(0) as i32).bind(stats["persistence"].as_i64().unwrap_or(0) as i32)
        .bind(stats["stability"].as_i64().unwrap_or(0) as i32).bind(stats["cpu"].as_i64().unwrap_or(1).max(1) as i32)
        .bind(stats["ram"].as_i64().unwrap_or(1).max(1) as i32).bind(stats["net"].as_i64().unwrap_or(1).max(1) as i32)
        .bind(weight as i32).bind(capability).execute(&mut *tx).await?;
    sqlx::query("UPDATE processes SET status='succeeded',result_code='COMPILED',result_message=$2 WHERE pid=$1")
        .bind(pid)
        .bind(format!("«{name}» compilado como fichero #{software_id}"))
        .execute(&mut *tx)
        .await?;
    tx.commit().await
}

/// An operation uses the resource and attribute snapshot recorded when it was started.
/// Later research cannot rewrite the outcome of an operation already in progress.
///
/// Winning the roll is only half of it: what a win *does* is the capability's `action_kind`,
/// read from the same frozen payload. A new capability is a row in `software_capabilities`;
/// only a new kind of consequence is a new arm here.
async fn modular_operation_effect(
    pool: &sqlx::PgPool,
    pid: i64,
    user: i64,
    victim: i64,
    victim_server: Option<i64>,
    soft: i64,
) -> Result<(), sqlx::Error> {
    let row = sqlx::query("SELECT process_payload FROM processes WHERE pid=$1")
        .bind(pid)
        .fetch_one(pool)
        .await?;
    let payload: serde_json::Value = row.get("process_payload");
    let capability = payload["capability"]
        .as_str()
        .unwrap_or_default()
        .to_owned();
    let kind = payload["action_kind"].as_str();
    let login = payload["grants_login"].as_str();
    let phase = |name: &str| -> (i32, i32, bool) {
        let value = &payload["phase_plan"][name];
        let probability = value["probability"].as_i64().unwrap_or(50) as i32;
        let roll = value["roll"].as_i64().unwrap_or(100) as i32;
        (probability, roll, roll <= probability)
    };
    let (execution_probability, execution_roll, execution) = phase("execution");
    let (penetration_probability, penetration_roll, penetration) = phase("penetration");
    let (detection_probability, detection_roll, detected) = phase("detection");
    let won = execution && penetration;
    let resistance = payload["defense_resistance"].as_i64().unwrap_or(0) as i32;
    for (name, probability, roll, outcome) in [
        (
            "execution",
            execution_probability,
            execution_roll,
            if execution { "succeeded" } else { "blocked" },
        ),
        (
            "penetration",
            penetration_probability,
            penetration_roll,
            if penetration { "succeeded" } else { "blocked" },
        ),
        (
            "detection",
            detection_probability,
            detection_roll,
            if detected { "detected" } else { "hidden" },
        ),
    ] {
        sqlx::query("INSERT INTO operation_phase_results(process_id,phase,probability,roll,outcome,details) VALUES($1,$2,$3,$4,$5,'{}') ON CONFLICT(process_id,phase) DO NOTHING")
            .bind(pid).bind(name).bind(probability).bind(roll).bind(outcome).execute(pool).await?;
    }
    // The defender is beaten but the world can still refuse the consequence -- a machine
    // that vanished, a disk with no room. That is a failure with its own reason, not a win
    // that quietly changed nothing.
    let refused = if won {
        match kind.as_deref() {
            Some("intrude") => {
                intrusion_effect(pool, user, victim, victim_server, login, resistance, &payload).await?
            }
            Some("infect") => infection_effect(pool, user, victim, victim_server, soft).await?,
            // A kind nobody has given a consequence yet resolves and leaves the world alone.
            _ => None,
        }
    } else {
        None
    };
    let (status, code, message) = match (won, refused, detected) {
        (true, Some((code, message)), _) => ("failed", code, message),
        (true, None, true) => (
            "succeeded",
            "SUCCEEDED_DETECTED",
            "operation completed, but the defender detected it",
        ),
        (true, None, false) => (
            "succeeded",
            "SUCCEEDED_STEALTH",
            "operation completed without detection",
        ),
        (false, _, true) if execution || penetration => (
            "failed",
            "PARTIAL_DETECTED",
            "la operación avanzó parcialmente y fue detectada",
        ),
        (false, _, false) if execution || penetration => (
            "failed",
            "PARTIAL",
            "la operación consiguió una fase, pero no acceso completo",
        ),
        (false, _, true) => (
            "failed",
            "BLOCKED_DETECTED",
            "defender blocked and detected the operation",
        ),
        (false, _, false) => ("failed", "BLOCKED", "defender resisted the operation"),
    };
    sqlx::query("UPDATE processes SET status=$2,result_code=$3,result_message=$4 WHERE pid=$1")
        .bind(pid)
        .bind(status)
        .bind(code)
        .bind(message)
        .execute(pool)
        .await?;
    // Every legacy action wrote a line on both machines. Here the victim's line is what
    // stealth buys: an operation their detectors never noticed leaves nothing on their log,
    // won or lost. The attacker's own machine always records what it did.
    let attacker_ip: Option<i64> = sqlx::query_scalar("SELECT game_ip FROM users WHERE id=$1")
        .bind(user)
        .fetch_one(pool)
        .await?;
    if let Some((target_ip, _)) = victim_ip(pool, victim, false, victim_server).await? {
        add_log(
            pool,
            user,
            false,
            &format!(
                "localhost ran {capability} against [{}] - {code}",
                dotted(target_ip)
            ),
        )
        .await?;
    }
    if detected {
        if let Some(attacker_ip) = attacker_ip {
            add_log(
                pool,
                victim,
                false,
                &format!(
                    "[{}] ran {capability} against localhost - {code}",
                    dotted(attacker_ip)
                ),
            )
            .await?;
        }
    }
    if let Some((target_ip, _)) = victim_ip(pool, victim, false, victim_server).await? {
        sqlx::query("INSERT INTO game_evidence(process_id,target_user_id,target_is_npc,action_code,real_origin_ip,real_identity,program_signature,integrity,concealment,state,attribution_confidence,visible_to_target,details) VALUES($1,$2,false,$3,$4,$5,$6,100,$7,$8,100,$9,$10) ON CONFLICT(process_id,action_code) DO NOTHING")
            .bind(pid).bind(victim).bind(&capability).bind(attacker_ip).bind(format!("user:{user}")).bind(&capability)
            .bind(if detected { 0 } else { 75 }).bind(if detected { "normal" } else { "hidden" }).bind(detected)
            .bind(serde_json::json!({"result_code":code,"target_ip":target_ip,"execution":execution,"penetration":penetration}))
            .execute(pool).await?;
    }
    Ok(())
}

/// A won intrusion opens the machine, exactly as cases 11 and 16 did. Which of the two it
/// behaves like is not decided here: a capability that names no login took the credentials
/// and enters as root, one that names a service user slipped in as that user without ever
/// learning the password.
async fn intrusion_effect(
    pool: &sqlx::PgPool,
    user: i64,
    victim: i64,
    victim_server: Option<i64>,
    grants_login: Option<&str>,
    resistance: i32,
    payload: &serde_json::Value,
) -> Result<Option<(&'static str, &'static str)>, sqlx::Error> {
    let Some((ip, _)) = victim_ip(pool, victim, false, victim_server).await? else {
        return Ok(Some(("TARGET_GONE", "the target machine no longer exists")));
    };
    let (login, password) = match grants_login {
        Some(login) => (login, "exploited".to_owned()),
        // Las operaciones modulares conceden una sesión de juego, no la contraseña real
        // del objetivo. El legado queda aislado mientras se migra su flujo de crack.
        None => ("root", "access-granted".to_owned()),
    };
    // ALREADY_LISTED, as in the legacy: a repeat refreshes the row instead of piling up, and
    // an exploit never downgrades a machine already held under its real password.
    let held: Option<String> =
        sqlx::query_scalar("SELECT password FROM compromised_lists WHERE user_id=$1 AND ip=$2")
            .bind(user)
            .bind(ip)
            .fetch_optional(pool)
            .await?;
    match held.as_deref() {
        Some(current) if current != "exploited" && password == "exploited" => {}
        Some(_) => {
            sqlx::query("UPDATE compromised_lists SET login=$3,password=$4,hacked_at=now() WHERE user_id=$1 AND ip=$2")
                .bind(user).bind(ip).bind(login).bind(&password).execute(pool).await?;
        }
        None => {
            sqlx::query(
                "INSERT INTO compromised_lists(user_id,ip,login,password) VALUES($1,$2,$3,$4)",
            )
            .bind(user)
            .bind(ip)
            .bind(login)
            .bind(&password)
            .execute(pool)
            .await?;
        }
    }
    // A machine already open by password stays open by password: an exploit on top of it
    // adds nothing, and marking it via_exploit would take access away.
    sqlx::query("INSERT INTO internet_access(user_id,ip,via_exploit) VALUES($1,$2,$3) ON CONFLICT(user_id,ip) DO UPDATE SET via_exploit=internet_access.via_exploit AND excluded.via_exploit,granted_at=now()")
        .bind(user).bind(ip).bind(grants_login.is_some()).execute(pool).await?;
    // El acceso táctico es la fuente de permisos; la fila heredada anterior solo mantiene
    // compatible el navegador ya existente. No se copia ninguna contraseña real aquí.
    let scope = payload["access_scope"].as_str().unwrap_or("service");
    let permissions = payload["granted_permissions"].clone();
    sqlx::query("INSERT INTO machine_accesses(attacker_id,target_user_id,target_is_npc,target_ip,identity_name,origin,privilege,scope,permissions) VALUES($1,$2,false,$3,$4,'operation',$5,$5,$6) ON CONFLICT(attacker_id,target_ip) DO UPDATE SET identity_name=EXCLUDED.identity_name,privilege=EXCLUDED.privilege,scope=EXCLUDED.scope,permissions=EXCLUDED.permissions,status='active',obtained_at=now()")
        .bind(user).bind(victim).bind(ip).bind(login).bind(scope).bind(permissions).execute(pool).await?;
    // The legacy paid for what stood in the way -- firewall and cracker versions. Here that
    // is the defence actually beaten, so a defended target is worth more than an open one.
    sqlx::query("UPDATE users_stats SET exp=exp+$2,hack_count=hack_count+1 WHERE uid=$1")
        .bind(user)
        .bind(resistance.max(1) as i64)
        .execute(pool)
        .await?;
    Ok(None)
}

/// A won infection leaves the program running on the victim's machine, hidden. The legacy
/// needed two steps for this -- upload, then install -- because it had no way to say "this
/// program plants itself"; a capability of kind `infect` is that way.
///
/// It still needs the machine to be open first. Planting something inside a machine nobody
/// got into would make `intrude` optional, and the two kinds are meant to chain.
async fn infection_effect(
    pool: &sqlx::PgPool,
    user: i64,
    victim: i64,
    victim_server: Option<i64>,
    soft: i64,
) -> Result<Option<(&'static str, &'static str)>, sqlx::Error> {
    let Some((ip, _)) = victim_ip(pool, victim, false, victim_server).await? else {
        return Ok(Some(("TARGET_GONE", "the target machine no longer exists")));
    };
    let open: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM internet_access WHERE user_id=$1 AND ip=$2)",
    )
    .bind(user)
    .bind(ip)
    .fetch_one(pool)
    .await?;
    if !open {
        return Ok(Some((
            "NO_ACCESS",
            "the target machine is not open: intrude before infecting",
        )));
    }
    let Some(f)=sqlx::query("SELECT soft_name,soft_version,soft_size,soft_ram,soft_type,licensed_to FROM software WHERE id=$1 AND user_id=$2 AND NOT is_npc").bind(soft).bind(user).fetch_optional(pool).await? else {
        return Ok(Some(("PAYLOAD_GONE", "the program that carried the operation is gone")));
    };
    let name: String = f.get("soft_name");
    let version: i32 = f.get("soft_version");
    let size: i32 = f.get("soft_size");
    let ram: i32 = f.get("soft_ram");
    // El virus se planta en el equipo en el que entraste, no en el que mejor le venga: esa
    // maquina la eligio el jugador al atacarla. Si el proceso no la dijo —lo de antes de que
    // un proceso apuntara a un nodo— vale la principal, que es a la que apuntaba de hecho.
    let server = match victim_server {
        Some(machine) => Some(machine),
        None => sqlx::query_scalar(
            "SELECT server_id FROM hardware WHERE user_id=$1 AND NOT is_npc AND is_gateway LIMIT 1",
        )
        .bind(victim)
        .fetch_optional(pool)
        .await?,
    };
    let Some(server) = server else {
        return Ok(Some(("TARGET_GONE", "the target machine no longer exists")));
    };
    // The copy has to fit on the victim's disk and stay loaded in their RAM: the same two
    // checks a transfer and an install go through. A machine with no room is not infected,
    // and the operation says so instead of claiming a silent win.
    if !fits_on(pool, server, size as i64).await? {
        return Ok(Some((
            "NO_DISK",
            "the target machine had no room for the payload",
        )));
    }
    let total_ram: Option<f32> =
        sqlx::query_scalar("SELECT sum(ram)::real FROM hardware WHERE user_id=$1 AND NOT is_npc")
            .bind(victim)
            .fetch_one(pool)
            .await?;
    let used_ram:Option<i64>=sqlx::query_scalar("SELECT sum(s.soft_ram)::bigint FROM software s JOIN software_running r ON r.soft_id=s.id WHERE r.user_id=$1 AND NOT r.is_npc").bind(victim).fetch_one(pool).await?;
    if (total_ram.unwrap_or(0.0) as i64) < used_ram.unwrap_or(0) + ram as i64 {
        return Ok(Some((
            "NO_RAM",
            "the target machine could not keep the payload running",
        )));
    }
    // Hidden, because a virus the owner can see on their own file list is not a virus. Their
    // seeker is what finds it, the same as any other hidden file.
    let planted:i64=sqlx::query_scalar("INSERT INTO software(user_id,soft_name,soft_version,soft_size,soft_ram,soft_type,soft_hidden,original_from,licensed_to,server_id) VALUES($1,$2,$3,$4,$5,$6,true,$7,$8,$9) RETURNING id")
        .bind(victim).bind(&name).bind(version).bind(size).bind(ram).bind(f.get::<i16,_>("soft_type")).bind(soft).bind(f.get::<i64,_>("licensed_to")).bind(server).fetch_one(pool).await?;
    sqlx::query("INSERT INTO software_running(soft_id,user_id,is_npc) VALUES($1,$2,false) ON CONFLICT DO NOTHING")
        .bind(planted).bind(victim).execute(pool).await?;
    // virus_id is where the legacy remembered whose virus sits on a machine. It only means
    // anything next to the row for that machine, so an infection with no listing sets none.
    sqlx::query("UPDATE compromised_lists SET virus_id=$3 WHERE user_id=$1 AND ip=$2")
        .bind(user)
        .bind(ip)
        .bind(planted)
        .execute(pool)
        .await?;
    Ok(None)
}

/// La direccion y la llave de **la maquina que se ataco**, no las de su dueno.
///
/// Aqui vivia media promesa sin cumplir. Esta funcion preguntaba «¿NPC o jugador?» y devolvia
/// lo del propietario: su `game_ip` y su `game_pass`. Mientras un jugador era una caja daba
/// igual, pero desde que tiene varias maquinas con direccion y llave propias, **atacar la
/// segunda y que el efecto aterrizara en la principal era sencillamente actuar sobre otra
/// maquina**. Lo arrastraban los nueve efectos que resuelven una victima: el crackeo, el
/// escaneo, el analisis, la edicion de bitacora, la intrusion, la infeccion, el exploit y las
/// dos operaciones modulares.
///
/// El proceso ya sabe a que nodo iba —`victim_server`, que escribe quien lo encola— asi que la
/// pregunta deja de ser de que tipo es el dueno y pasa a ser cual de sus equipos.
///
/// `is_npc` sigue en la firma para los procesos de antes de que existiera la columna, que no
/// tienen nodo y solo se pueden resolver como se resolvian. Es el camino de atras, no el
/// principal.
async fn victim_ip(
    pool: &sqlx::PgPool,
    victim: i64,
    is_npc: bool,
    victim_server: Option<i64>,
) -> Result<Option<(i64, String)>, sqlx::Error> {
    if let Some(machine) = victim_server {
        return Ok(sqlx::query(
            "SELECT public_ip AS ip,machine_pass AS pass FROM hardware \
             WHERE server_id=$1 AND public_ip IS NOT NULL",
        )
        .bind(machine)
        .fetch_optional(pool)
        .await?
        .map(|r| (r.get("ip"), r.get("pass"))));
    }

    let sql = if is_npc {
        "SELECT npc_ip AS ip,npc_pass AS pass FROM npc WHERE id=$1"
    } else {
        "SELECT game_ip AS ip,game_pass AS pass FROM users WHERE id=$1"
    };
    Ok(sqlx::query(sql)
        .bind(victim)
        .fetch_optional(pool)
        .await?
        .map(|r| (r.get("ip"), r.get("pass"))))
}

#[cfg(test)]
mod tests {
    use super::modification_is_partial;

    #[test]
    fn near_miss_is_partial_but_does_not_turn_a_failed_roll_into_success() {
        assert!(modification_is_partial(false, 50, 51));
        assert!(modification_is_partial(false, 50, 60));
        assert!(!modification_is_partial(false, 50, 61));
        assert!(!modification_is_partial(true, 50, 1));
    }
}

/// Case 11 of completeProcess. A successful brute force lands the victim's credentials in
/// the player's compromised list, opens the machine and pays experience.
///
/// **Lo que se apunta es la maquina que se ataco, no su dueno.** Antes salia de `victim_ip`,
/// que resolvia al propietario: atacaras la direccion que atacaras, en la lista quedaba la de
/// la principal y la contrasena de la cuenta. Crackear una era crackearlo todo, y las
/// direcciones separadas no servian de nada.
async fn hack_effect(
    pool: &sqlx::PgPool,
    user: i64,
    victim: i64,
    is_npc: bool,
    victim_server: Option<i64>,
) -> Result<(), sqlx::Error> {
    let Some((ip, pass)) = victim_ip(pool, victim, is_npc, victim_server).await? else {
        return Ok(());
    };
    // addToList($_SESSION['id'], $ip, 'root', $pass, '0', '0'). The legacy refused a repeat
    // with ALREADY_LISTED; compromised_lists has no unique key, so rather than pile up
    // duplicates a repeat hack refreshes the row already there.
    let known: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM compromised_lists WHERE user_id=$1 AND ip=$2)",
    )
    .bind(user)
    .bind(ip)
    .fetch_one(pool)
    .await?;
    if known {
        sqlx::query(
            "UPDATE compromised_lists SET password=$3,hacked_at=now() WHERE user_id=$1 AND ip=$2",
        )
        .bind(user)
        .bind(ip)
        .bind(&pass)
        .execute(pool)
        .await?;
    } else {
        sqlx::query(
            "INSERT INTO compromised_lists(user_id,ip,login,password) VALUES($1,$2,'root',$3)",
        )
        .bind(user)
        .bind(ip)
        .bind(&pass)
        .execute(pool)
        .await?;
    }
    // The crack is also what opens the machine, so the player need not log in again.
    sqlx::query("INSERT INTO internet_access(user_id,ip,via_exploit) VALUES($1,$2,false) ON CONFLICT(user_id,ip) DO UPDATE SET granted_at=now()")
        .bind(user).bind(ip).execute(pool).await?;
    // exp_add('HACK', hasher version + victim cracker version), as in the legacy.
    let exp:i64=sqlx::query_scalar("SELECT COALESCE(sum(soft_version),0)::bigint FROM software WHERE user_id=$1 AND soft_type IN (1,2) AND is_npc=$2")
        .bind(victim).bind(is_npc).fetch_one(pool).await?;
    sqlx::query("UPDATE users_stats SET exp=exp+$2,hack_count=hack_count+1 WHERE uid=$1")
        .bind(user)
        .bind(exp)
        .execute(pool)
        .await?;
    // Every action in completeProcess wrote two log lines, one on each side, through
    // LogVPC::logText('LOGIN', ...). The victim sees who came in, the attacker sees where
    // they went, and the pair is what makes log editing worth doing at all.
    if let Some(attacker_ip) = sqlx::query_scalar::<_, i64>("SELECT game_ip FROM users WHERE id=$1")
        .bind(user)
        .fetch_optional(pool)
        .await?
    {
        add_log(
            pool,
            victim,
            is_npc,
            &format!("[{}] logged in as root", dotted(attacker_ip)),
        )
        .await?;
        add_log(
            pool,
            user,
            false,
            &format!("localhost logged in to [{}] as root", dotted(ip)),
        )
        .await?;
    }
    Ok(())
}

/// Port of LogVPC::addLog. The legacy stamped every line with the minute it happened and
/// prepended it, so a machine keeps one log row with the newest entry on top.
async fn add_log(
    pool: &sqlx::PgPool,
    owner: i64,
    is_npc: bool,
    text: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE log SET text=to_char(now(),'YYYY-MM-DD HH24:MI')||' - '||$3||chr(10)||text WHERE user_id=$1 AND is_npc=$2")
        .bind(owner).bind(is_npc).bind(text).execute(pool).await?;
    Ok(())
}

/// long2ip. Game IPs travel as the 32-bit integers the legacy stored, but a log line is
/// read by a person.
fn dotted(ip: i64) -> String {
    format!(
        "{}.{}.{}.{}",
        (ip >> 24) & 255,
        (ip >> 16) & 255,
        (ip >> 8) & 255,
        ip & 255
    )
}

/// Case 8 of completeProcess. The replacement text waited in log_edit while the process
/// ran; finishing it is what actually overwrites the victim's log.
///
/// There is no case 9: wiping a log is this same case with an empty text, which is why
/// D_LOG needs nothing of its own.
async fn edit_log_effect(
    pool: &sqlx::PgPool,
    pid: i64,
    victim: i64,
    is_npc: bool,
) -> Result<(), sqlx::Error> {
    let Some(text) = sqlx::query_scalar::<_, String>(
        "SELECT log_text FROM log_edit WHERE process_id=$1 ORDER BY id DESC LIMIT 1",
    )
    .bind(pid)
    .fetch_optional(pool)
    .await?
    else {
        return Ok(());
    };
    sqlx::query("UPDATE log SET text=$3 WHERE user_id=$1 AND is_npc=$2")
        .bind(victim)
        .bind(is_npc)
        .bind(text)
        .execute(pool)
        .await?;
    // deleteTmpLog: the pending text has been spent, and leaving it behind would pile up
    // a row per edit forever.
    sqlx::query("DELETE FROM log_edit WHERE process_id=$1")
        .bind(pid)
        .execute(pool)
        .await?;
    Ok(())
}

/// Case 15 of completeProcess. The legacy only set $_SESSION['PORT_SCAN'] so the next
/// render could list the open ports; persisted, the scan outlives the request.
async fn scan_effect(
    pool: &sqlx::PgPool,
    user: i64,
    victim: i64,
    is_npc: bool,
    victim_server: Option<i64>,
) -> Result<(), sqlx::Error> {
    let Some((ip, _)) = victim_ip(pool, victim, is_npc, victim_server).await? else {
        return Ok(());
    };
    sqlx::query("INSERT INTO internet_scan(user_id,ip) VALUES($1,$2) ON CONFLICT(user_id,ip) DO UPDATE SET scanned_at=now()")
        .bind(user).bind(ip).execute(pool).await?;
    Ok(())
}

/// Cases 1 and 2 of completeProcess, download and upload. Both copy a file between two
/// machines, so they differ only in which end is the source.
///
/// The legacy checked here rather than at the start: a copy that no longer fits, or that
/// duplicates something already held, is dropped silently at this point.
async fn transfer_effect(
    pool: &sqlx::PgPool,
    user: i64,
    victim: i64,
    victim_is_npc: bool,
    victim_server: Option<i64>,
    soft: i64,
    action: i16,
) -> Result<(), sqlx::Error> {
    let download = action == 1;
    let (from, from_npc, to, to_npc) = if download {
        (victim, victim_is_npc, user, false)
    } else {
        (user, false, victim, victim_is_npc)
    };
    let Some(f)=sqlx::query("SELECT soft_name,soft_version,soft_size,soft_ram,soft_type,licensed_to FROM software WHERE id=$1 AND user_id=$2 AND is_npc=$3").bind(soft).bind(from).bind(from_npc).fetch_optional(pool).await? else {
        return Ok(());
    };
    let name: String = f.get("soft_name");
    let version: i32 = f.get("soft_version");
    let size: i32 = f.get("soft_size");
    let ram: i32 = f.get("soft_ram");
    let soft_type: i16 = f.get("soft_type");
    let licensed: i64 = f.get("licensed_to");
    // "SELECT id FROM software WHERE ... softType AND softVersion AND softName": the same
    // file twice is not worth the disk.
    let held:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM software WHERE user_id=$1 AND is_npc=$2 AND soft_type=$3 AND soft_version=$4 AND soft_name=$5 AND NOT soft_hidden)").bind(to).bind(to_npc).bind(soft_type).bind(version).bind(&name).fetch_one(pool).await?;
    if held {
        return Ok(());
    }
    // calculateHDDUsage: el fichero cae en un disco concreto, o no cae.
    let Some(server) = landing_disk(pool, to, to_npc, size as i64).await? else {
        return Ok(());
    };
    // originalFrom keeps the trail back to the file this copy was taken from.
    sqlx::query("INSERT INTO software(user_id,soft_name,soft_version,soft_size,soft_ram,soft_type,is_npc,original_from,licensed_to,server_id) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)")
        .bind(to).bind(&name).bind(version).bind(size).bind(ram).bind(soft_type).bind(to_npc).bind(soft).bind(licensed).bind(server).execute(pool).await?;
    let verb = if download { "downloaded" } else { "uploaded" };
    log_pair(
        pool,
        user,
        victim,
        victim_is_npc,
        victim_server,
        verb,
        &format!("{name}{version}"),
    )
    .await?;
    Ok(())
}

/// Case 3 of completeProcess. A remote delete takes the file away for good.
async fn delete_effect(
    pool: &sqlx::PgPool,
    user: i64,
    victim: i64,
    victim_is_npc: bool,
    victim_server: Option<i64>,
    soft: i64,
) -> Result<(), sqlx::Error> {
    let Some(f) = sqlx::query(
        "SELECT soft_name,soft_version FROM software WHERE id=$1 AND user_id=$2 AND is_npc=$3",
    )
    .bind(soft)
    .bind(victim)
    .bind(victim_is_npc)
    .fetch_optional(pool)
    .await?
    else {
        return Ok(());
    };
    let label = format!(
        "{}{}",
        f.get::<String, _>("soft_name"),
        f.get::<i32, _>("soft_version")
    );
    sqlx::query("DELETE FROM software WHERE id=$1")
        .bind(soft)
        .execute(pool)
        .await?;
    log_pair(pool, user, victim, victim_is_npc, victim_server, "deleted", &label).await?;
    Ok(())
}

/// The two-sided log line every file action in completeProcess wrote: the victim reads it
/// as coming from an address, the attacker as going out to one.
async fn log_pair(
    pool: &sqlx::PgPool,
    user: i64,
    victim: i64,
    victim_is_npc: bool,
    victim_server: Option<i64>,
    verb: &str,
    file: &str,
) -> Result<(), sqlx::Error> {
    let attacker_ip: Option<i64> = sqlx::query_scalar("SELECT game_ip FROM users WHERE id=$1")
        .bind(user)
        .fetch_one(pool)
        .await?;
    let Some((victim_ip, _)) = victim_ip(pool, victim, victim_is_npc, victim_server).await? else {
        return Ok(());
    };
    if let Some(attacker_ip) = attacker_ip {
        add_log(
            pool,
            victim,
            victim_is_npc,
            &format!("[{}] {verb} file {file} at localhost", dotted(attacker_ip)),
        )
        .await?;
        add_log(
            pool,
            user,
            false,
            &format!("localhost {verb} file {file} at [{}]", dotted(victim_ip)),
        )
        .await?;
    }
    Ok(())
}

/// isVirus: the five types the legacy would not let you uninstall, because a virus is
/// meant to be dug out with an antivirus rather than clicked away.
fn is_virus(soft_type: i16) -> bool {
    matches!(soft_type, 8 | 9 | 10 | 20 | 29)
}

/// isExecutable. A virus, a text file, a folder and a torrent are the things that have
/// nothing to run.
fn is_executable(soft_type: i16) -> bool {
    !is_virus(soft_type) && !matches!(soft_type, 17 | 30 | 31)
}

/// Which machine a software process acts on. victim_id 0 means the player's own, which is
/// how the legacy told a local action from a remote one.
fn target_of(user: i64, victim: i64, victim_is_npc: bool) -> (i64, bool) {
    if victim == 0 {
        (user, false)
    } else {
        (victim, victim_is_npc)
    }
}

/// Case 4 of completeProcess. A file is buried under the version of the best hider the
/// player has running; SEEK later has to beat that number to dig it out.
async fn hide_effect(
    pool: &sqlx::PgPool,
    user: i64,
    victim: i64,
    victim_is_npc: bool,
    soft: i64,
) -> Result<(), sqlx::Error> {
    let (owner, owner_is_npc) = target_of(user, victim, victim_is_npc);
    let Some(f) =
        sqlx::query("SELECT soft_type FROM software WHERE id=$1 AND user_id=$2 AND is_npc=$3")
            .bind(soft)
            .bind(owner)
            .bind(owner_is_npc)
            .fetch_optional(pool)
            .await?
    else {
        return Ok(());
    };
    // "You cant hide a folder", and a running program cannot be hidden either.
    let soft_type: i16 = f.get("soft_type");
    if soft_type == 31 || soft_type == 29 {
        return Ok(());
    }
    let installed: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM software_running WHERE soft_id=$1)")
            .bind(soft)
            .fetch_one(pool)
            .await?;
    if installed {
        return Ok(());
    }
    let hider:Option<i32>=sqlx::query_scalar("SELECT max(s.soft_version) FROM software s JOIN software_running r ON r.soft_id=s.id WHERE s.user_id=$1 AND NOT s.is_npc AND s.soft_type=5").bind(user).fetch_one(pool).await?;
    let Some(hider) = hider else {
        return Ok(());
    };
    sqlx::query("UPDATE software SET soft_hidden=TRUE,soft_hidden_with=$2 WHERE id=$1")
        .bind(soft)
        .bind(hider)
        .execute(pool)
        .await?;
    Ok(())
}

/// Case 5 of completeProcess. The seeker has to be at least as good as the hider that
/// buried the file, which is the whole point of keeping soft_hidden_with around.
async fn seek_effect(
    pool: &sqlx::PgPool,
    user: i64,
    victim: i64,
    victim_is_npc: bool,
    soft: i64,
) -> Result<(), sqlx::Error> {
    let (owner, owner_is_npc) = target_of(user, victim, victim_is_npc);
    let seeker:Option<i32>=sqlx::query_scalar("SELECT max(s.soft_version) FROM software s JOIN software_running r ON r.soft_id=s.id WHERE s.user_id=$1 AND NOT s.is_npc AND s.soft_type=6").bind(user).fetch_one(pool).await?;
    let Some(seeker) = seeker else {
        return Ok(());
    };
    sqlx::query("UPDATE software SET soft_hidden=FALSE,soft_hidden_with=0 WHERE id=$1 AND user_id=$2 AND is_npc=$3 AND soft_hidden AND soft_hidden_with<=$4")
        .bind(soft).bind(owner).bind(owner_is_npc).bind(seeker).execute(pool).await?;
    Ok(())
}

/// Case 13 of completeProcess. Loading a program is what RAM is for: the legacy refused
/// the install when calculateRamUsage had nothing left to give.
async fn install_effect(
    pool: &sqlx::PgPool,
    user: i64,
    victim: i64,
    victim_is_npc: bool,
    soft: i64,
) -> Result<(), sqlx::Error> {
    let (owner, owner_is_npc) = target_of(user, victim, victim_is_npc);
    let Some(f)=sqlx::query("SELECT soft_type,soft_ram,soft_hidden FROM software WHERE id=$1 AND user_id=$2 AND is_npc=$3").bind(soft).bind(owner).bind(owner_is_npc).fetch_optional(pool).await? else {
        return Ok(());
    };
    let soft_type: i16 = f.get("soft_type");
    let need: i32 = f.get("soft_ram");
    if !is_executable(soft_type) || f.get::<bool, _>("soft_hidden") {
        return Ok(());
    }
    let already: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM software_running WHERE soft_id=$1)")
            .bind(soft)
            .fetch_one(pool)
            .await?;
    if already {
        return Ok(());
    }
    // calculateRamUsage, ahora de verdad **de la maquina**: el comentario decia eso desde el
    // principio pero las dos consultas sumaban la flota entera, asi que un programa cabia en
    // un equipo pequeno mientras el dueno tuviera memoria en otro. Un programa corre donde
    // esta guardado, y la memoria que gasta es la de ese equipo.
    let total: Option<f32> = sqlx::query_scalar(
        "SELECT h.ram::real FROM hardware h JOIN software s ON s.server_id = h.server_id WHERE s.id = $1",
    )
    .bind(soft)
    .fetch_optional(pool)
    .await?
    .flatten();
    let used: Option<i64> = sqlx::query_scalar(
        "SELECT sum(s.soft_ram)::bigint FROM software s JOIN software_running r ON r.soft_id = s.id WHERE s.server_id = (SELECT server_id FROM software WHERE id = $1)",
    )
    .bind(soft)
    .fetch_one(pool)
    .await?;
    if (total.unwrap_or(0.0) as i64) < used.unwrap_or(0) + need as i64 {
        return Ok(());
    }
    sqlx::query("INSERT INTO software_running(soft_id,user_id,is_npc) VALUES($1,$2,$3) ON CONFLICT DO NOTHING")
        .bind(soft).bind(owner).bind(owner_is_npc).execute(pool).await?;
    Ok(())
}

/// Case 14 of completeProcess. Everything but a virus can be unloaded again.
async fn uninstall_effect(
    pool: &sqlx::PgPool,
    user: i64,
    victim: i64,
    victim_is_npc: bool,
    soft: i64,
) -> Result<(), sqlx::Error> {
    let (owner, owner_is_npc) = target_of(user, victim, victim_is_npc);
    let Some(f) =
        sqlx::query("SELECT soft_type FROM software WHERE id=$1 AND user_id=$2 AND is_npc=$3")
            .bind(soft)
            .bind(owner)
            .bind(owner_is_npc)
            .fetch_optional(pool)
            .await?
    else {
        return Ok(());
    };
    if is_virus(f.get::<i16, _>("soft_type")) {
        return Ok(());
    }
    sqlx::query("DELETE FROM software_running WHERE soft_id=$1")
        .bind(soft)
        .execute(pool)
        .await?;
    Ok(())
}

/// Case 25 of completeProcess. A new address is the only way out of an IP that half the
/// server has written down, so the legacy checked the draw against every machine before
/// taking it.
///
/// The legacy also billed this to a bank account, priced off uptime by Player::ip_info.
/// The port has neither the uptime column nor the reset timers that price depends on, so
/// the reset is free here until those exist.
async fn reset_ip_effect(pool: &sqlx::PgPool, user: i64) -> Result<(), sqlx::Error> {
    // El sorteo y la comprobacion de que este libre viven en `random_free_ip()`, para que
    // el worker y el alta de usuarios repartan direcciones con la misma regla.
    let candidate: i64 = sqlx::query_scalar("SELECT random_free_ip()")
        .fetch_one(pool)
        .await?;

    // **La direccion es de la maquina principal, y `game_ip` la sigue.** Antes esto cambiaba
    // solo la columna del usuario, y desde que se resuelve por la maquina eso dejaba al
    // jugador respondiendo en la direccion vieja mientras la nueva no llevaba a ninguna parte.
    // Las dos escrituras van juntas o no van: si se separaran, la identidad y la puerta
    // apuntarian a sitios distintos.
    let mut tx = pool.begin().await?;

    sqlx::query(
        "UPDATE hardware SET public_ip=$2 WHERE user_id=$1 AND NOT is_npc AND is_gateway",
    )
    .bind(user)
    .bind(candidate)
    .execute(&mut *tx)
    .await?;

    sqlx::query("UPDATE users SET game_ip=$2 WHERE id=$1")
        .bind(user)
        .bind(candidate)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    Ok(())
}

/// Case 26 of completeProcess, and the answer to being cracked. Changing the password is
/// not only a new string: the legacy went through every list holding the old one and blanked
/// it to 'unknown', so a stolen credential stops working the moment its owner reacts.
///
/// Entries marked 'exploited' were left alone, because those were not taken by guessing
/// the password in the first place.
async fn reset_password_effect(pool: &sqlx::PgPool, user: i64) -> Result<(), sqlx::Error> {
    let Some(ip) = sqlx::query_scalar::<_, i64>("SELECT game_ip FROM users WHERE id=$1")
        .bind(user)
        .fetch_optional(pool)
        .await?
    else {
        return Ok(());
    };
    // **La llave es de la maquina principal, y `game_pass` la sigue**, igual que la direccion
    // desde la fase 3. Las dos escrituras van juntas: si se separaran, la cuenta y la puerta
    // dirian cosas distintas. Cambiar la de un equipo concreto es lo que falta, y necesita
    // que el jugador pueda elegir cual.
    let nueva: String =
        sqlx::query_scalar("SELECT substr(md5(random()::text),1,8)")
            .fetch_one(pool)
            .await?;

    let mut tx = pool.begin().await?;

    sqlx::query(
        "UPDATE hardware SET machine_pass=$2 WHERE user_id=$1 AND NOT is_npc AND is_gateway",
    )
    .bind(user)
    .bind(&nueva)
    .execute(&mut *tx)
    .await?;

    sqlx::query("UPDATE users SET game_pass=$2 WHERE id=$1")
        .bind(user)
        .bind(&nueva)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;
    sqlx::query("UPDATE compromised_lists SET password='unknown',virus_id=0 WHERE ip=$1 AND password<>'exploited'")
        .bind(ip).execute(pool).await?;
    Ok(())
}

/// Case 17 of completeProcess. Research is the upgrade path that does not need a victim:
/// you improve what you already own rather than steal a better copy. Each run raises the
/// version by one and leaves a row in research_history.
async fn research_effect(pool: &sqlx::PgPool, user: i64, soft: i64) -> Result<(), sqlx::Error> {
    let Some(f)=sqlx::query("SELECT soft_name,soft_version,soft_type FROM software WHERE id=$1 AND user_id=$2 AND NOT is_npc").bind(soft).bind(user).fetch_optional(pool).await? else {
        return Ok(());
    };
    let version: i32 = f.get::<i32, _>("soft_version") + 1;
    sqlx::query("UPDATE software SET soft_version=$1 WHERE id=$2")
        .bind(version)
        .bind(soft)
        .execute(pool)
        .await?;
    sqlx::query("INSERT INTO research_history(user_id,software_id,software_type,new_version,software_name) VALUES($1,$2,$3,$4,$5)")
        .bind(user).bind(soft).bind(f.get::<i16,_>("soft_type")).bind(version).bind(f.get::<String,_>("soft_name")).execute(pool).await?;
    Ok(())
}

/// Case 23 of completeProcess. An analyzer reads a machine's CPU and memory, and how close
/// the reading lands depends on the tool: the legacy gave a v10 fifty percent accuracy, a
/// v20 eighty and a v30 the exact figure.
///
/// It switched on the version being exactly 10, 20 or 30 and killed the request otherwise,
/// which any researched analyzer would trip. Reading it as thresholds keeps the same three
/// tiers without that cliff.
async fn analyze_effect(
    pool: &sqlx::PgPool,
    user: i64,
    victim: i64,
    victim_is_npc: bool,
    victim_server: Option<i64>,
    soft: i64,
) -> Result<(), sqlx::Error> {
    let Some(version)=sqlx::query_scalar::<_,i32>("SELECT soft_version FROM software WHERE id=$1 AND user_id=$2 AND NOT is_npc AND soft_type=16").bind(soft).bind(user).fetch_optional(pool).await? else {
        return Ok(());
    };
    let accuracy = if version >= 30 {
        1.0
    } else if version >= 20 {
        0.8
    } else {
        0.5
    };
    let Some((ip, _)) = victim_ip(pool, victim, victim_is_npc, victim_server).await? else {
        return Ok(());
    };
    let Some(hw)=sqlx::query("SELECT sum(cpu)::real AS cpu,sum(ram)::real AS ram FROM hardware WHERE user_id=$1 AND is_npc=$2").bind(victim).bind(victim_is_npc).fetch_optional(pool).await? else {
        return Ok(());
    };
    let cpu: f32 = hw.get::<Option<f32>, _>("cpu").unwrap_or(0.0);
    let ram: f32 = hw.get::<Option<f32>, _>("ram").unwrap_or(0.0);
    // The legacy nudged each bound inwards by a random fraction of the error band, so two
    // readings of the same machine never matched exactly.
    let jitter: f32 = sqlx::query_scalar::<_, f32>("SELECT (5+random()*45)::real")
        .fetch_one(pool)
        .await?;
    let span_cpu = cpu * (1.0 - accuracy);
    let span_ram = ram * (1.0 - accuracy);
    let cpu_min = (cpu - span_cpu + span_cpu / jitter).round() as i32;
    let cpu_max = (cpu + span_cpu - span_cpu / jitter).round() as i32;
    let ram_min = (ram - span_ram + span_ram / jitter).round() as i32;
    let ram_max = (ram + span_ram - span_ram / jitter).round() as i32;
    sqlx::query("INSERT INTO analyze_results(user_id,ip,cpu_min,cpu_max,ram_min,ram_max) VALUES($1,$2,$3,$4,$5,$6) ON CONFLICT(user_id,ip) DO UPDATE SET cpu_min=EXCLUDED.cpu_min,cpu_max=EXCLUDED.cpu_max,ram_min=EXCLUDED.ram_min,ram_max=EXCLUDED.ram_max,analyzed_at=now()")
        .bind(user).bind(ip).bind(cpu_min).bind(cpu_max).bind(ram_min).bind(ram_max).execute(pool).await?;
    Ok(())
}

/// Best version of a software type a machine has loaded, which is what getBestSoftware
/// asked for whenever it passed installed = 1.
async fn best_running(
    pool: &sqlx::PgPool,
    owner: i64,
    is_npc: bool,
    soft_type: i16,
) -> Result<i32, sqlx::Error> {
    sqlx::query_scalar("SELECT COALESCE(max(s.soft_version),0) FROM software s JOIN software_running r ON r.soft_id=s.id WHERE s.user_id=$1 AND s.is_npc=$2 AND s.soft_type=$3 AND NOT s.soft_hidden")
        .bind(owner).bind(is_npc).bind(soft_type).fetch_one(pool).await
}

/// Case 16 of completeProcess, the other way in. Rather than guessing the password you go
/// through a service, so an FTP or SSH exploit is measured against the victim's firewall
/// instead of their hasher.
///
/// Which exploits land decides who you become: ftp alone, ssh alone, or root with both.
/// The credential is filed as 'exploited' rather than a password, and that is exactly why
/// reset_password_effect leaves those rows alone -- there was never a password to change,
/// so changing one takes nothing away.
async fn exploit_effect(
    pool: &sqlx::PgPool,
    user: i64,
    victim: i64,
    victim_is_npc: bool,
    victim_server: Option<i64>,
) -> Result<bool, sqlx::Error> {
    let Some((ip, _)) = victim_ip(pool, victim, victim_is_npc, victim_server).await? else {
        return Ok(false);
    };
    let firewall = best_running(pool, victim, victim_is_npc, 4).await?;
    let ftp = best_running(pool, user, false, 13).await?;
    let ssh = best_running(pool, user, false, 14).await?;
    let login = match (ftp > 0 && ftp >= firewall, ssh > 0 && ssh >= firewall) {
        (true, true) => "root",
        (true, false) => "ftp",
        (false, true) => "ssh",
        // Neither exploit clears the firewall, so nothing opens.
        (false, false) => return Ok(false),
    };
    // ALREADY_LISTED: the legacy refused when the machine was already held under a real
    // password, and refused a repeat exploit that would not improve on the user it granted.
    let held: Option<String> =
        sqlx::query_scalar("SELECT password FROM compromised_lists WHERE user_id=$1 AND ip=$2")
            .bind(user)
            .bind(ip)
            .fetch_optional(pool)
            .await?;
    match held.as_deref() {
        Some("exploited") => {
            sqlx::query(
                "UPDATE compromised_lists SET login=$3,hacked_at=now() WHERE user_id=$1 AND ip=$2",
            )
            .bind(user)
            .bind(ip)
            .bind(login)
            .execute(pool)
            .await?;
        }
        Some(_) => return Ok(false),
        None => {
            sqlx::query("INSERT INTO compromised_lists(user_id,ip,login,password) VALUES($1,$2,$3,'exploited')")
                .bind(user).bind(ip).bind(login).execute(pool).await?;
        }
    }
    sqlx::query("INSERT INTO internet_access(user_id,ip,via_exploit) VALUES($1,$2,true) ON CONFLICT(user_id,ip) DO UPDATE SET via_exploit=true,granted_at=now()")
        .bind(user).bind(ip).execute(pool).await?;
    let cracker = best_running(pool, victim, victim_is_npc, 1).await?;
    sqlx::query("UPDATE users_stats SET exp=exp+$2,hack_count=hack_count+1 WHERE uid=$1")
        .bind(user)
        .bind((firewall + cracker) as i64)
        .execute(pool)
        .await?;
    if let Some(attacker_ip) = sqlx::query_scalar::<_, i64>("SELECT game_ip FROM users WHERE id=$1")
        .bind(user)
        .fetch_optional(pool)
        .await?
    {
        add_log(
            pool,
            victim,
            victim_is_npc,
            &format!("[{}] logged in as {login}", dotted(attacker_ip)),
        )
        .await?;
        add_log(
            pool,
            user,
            false,
            &format!("localhost logged in to [{}] as {login}", dotted(ip)),
        )
        .await?;
    }
    Ok(true)
}

/// Case 18 of completeProcess. Parking a file on the external drive, which is the point of
/// having one: it is not on the machine, so formatting or losing the machine leaves it.
///
/// The row keeps the source file's id rather than a fresh one, exactly as the legacy wrote
/// it. A parked file is the same file, somewhere else.
async fn park_effect(pool: &sqlx::PgPool, user: i64, soft: i64) -> Result<(), sqlx::Error> {
    let Some(f)=sqlx::query("SELECT soft_name,soft_version,soft_size,soft_ram,soft_type,licensed_to FROM software WHERE id=$1 AND user_id=$2 AND NOT is_npc").bind(soft).bind(user).fetch_optional(pool).await? else {
        return Ok(());
    };
    let size: i32 = f.get("soft_size");
    let already: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM software_external WHERE id=$1)")
            .bind(soft)
            .fetch_one(pool)
            .await?;
    if already {
        return Ok(());
    }
    // BAD_XHD: the drive's capacity against what is already parked on it.
    let capacity: Option<i64> =
        sqlx::query_scalar("SELECT sum(size)::bigint FROM hardware_external WHERE user_id=$1")
            .bind(user)
            .fetch_one(pool)
            .await?;
    let used: Option<i64> =
        sqlx::query_scalar("SELECT sum(soft_size)::bigint FROM software_external WHERE user_id=$1")
            .bind(user)
            .fetch_one(pool)
            .await?;
    if capacity.unwrap_or(0) < used.unwrap_or(0) + size as i64 {
        return Ok(());
    }
    sqlx::query("INSERT INTO software_external(id,user_id,soft_name,soft_version,soft_size,soft_ram,soft_type,licensed_to) VALUES($1,$2,$3,$4,$5,$6,$7,$8)")
        .bind(soft).bind(user).bind(f.get::<String,_>("soft_name")).bind(f.get::<i32,_>("soft_version")).bind(size).bind(f.get::<i32,_>("soft_ram")).bind(f.get::<i16,_>("soft_type")).bind(f.get::<i64,_>("licensed_to")).execute(pool).await?;
    Ok(())
}

/// Case 19. Bringing a parked file back onto the machine, refused when the disk has no
/// room or a copy is already sitting there.
async fn unpark_effect(pool: &sqlx::PgPool, user: i64, soft: i64) -> Result<(), sqlx::Error> {
    let Some(f)=sqlx::query("SELECT soft_name,soft_version,soft_size,soft_ram,soft_type,licensed_to FROM software_external WHERE id=$1 AND user_id=$2").bind(soft).bind(user).fetch_optional(pool).await? else {
        return Ok(());
    };
    let name: String = f.get("soft_name");
    let version: i32 = f.get("soft_version");
    let size: i32 = f.get("soft_size");
    let held:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM software WHERE user_id=$1 AND NOT is_npc AND soft_name=$2 AND soft_version=$3)").bind(user).bind(&name).bind(version).fetch_one(pool).await?;
    if held {
        return Ok(());
    }
    // BAD_HDD, la misma comprobacion de disco que pasa toda copia que aterriza en una maquina.
    let Some(server) = landing_disk(pool, user, false, size as i64).await? else {
        return Ok(());
    };
    sqlx::query("INSERT INTO software(user_id,soft_name,soft_version,soft_size,soft_ram,soft_type,licensed_to,server_id) VALUES($1,$2,$3,$4,$5,$6,$7,$8)")
        .bind(user).bind(&name).bind(version).bind(size).bind(f.get::<i32,_>("soft_ram")).bind(f.get::<i16,_>("soft_type")).bind(f.get::<i64,_>("licensed_to")).bind(server).execute(pool).await?;
    Ok(())
}

/// Case 20. Clearing a file off the drive, which is the only way to make room on it.
async fn discard_effect(pool: &sqlx::PgPool, user: i64, soft: i64) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM software_external WHERE id=$1 AND user_id=$2")
        .bind(soft)
        .bind(user)
        .execute(pool)
        .await?;
    Ok(())
}

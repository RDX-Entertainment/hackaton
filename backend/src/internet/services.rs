use sqlx::PgPool;

use crate::error::ApiError;

use super::{
    dtos::{
        AnalyzeStartedResponse,
        BookmarkCreatedResponse,
        BookmarkResponse,
        HardwareAnalysisResponse,
        InternetHackRequest,
        InternetHackResponse,
        InternetHistoryResponse,
        InternetLoginRequest,
        InternetLoginResponse,
        BankRef,
        InternetNavigateResponse,
        InternetTargetResponse,
        IpRequest,
        LogEditRequest,
        LogEditStartedResponse,
        RemoteLogResponse,
    },
    models::Remote,
    repo,
    webscript,
};

const ANALYZE_SECONDS: i32 = 60;

/// El primer acceso otorga el nivel 2: pedirlo tambien en la practica cerraba el circuito.
/// El alumno de nivel 1 puede usar fuerza bruta en su campo de tiro; el mundo sigue igual.
fn can_start_hack(level: i16, required: i16, action: i16, own_practice: bool) -> bool {
    level >= required || (level >= 1 && action == 11 && own_practice)
}

#[cfg(test)]
mod tutorial_tests {
    use super::can_start_hack;

    #[test]
    fn first_access_is_possible_only_on_the_students_practice_target() {
        assert!(can_start_hack(1, 2, 11, true));
        assert!(!can_start_hack(1, 2, 11, false));
        assert!(!can_start_hack(0, 2, 11, true));
    }

    #[test]
    fn practice_does_not_unlock_scanning_or_change_regular_certifications() {
        assert!(!can_start_hack(1, 3, 15, true));
        assert!(can_start_hack(2, 2, 11, false));
        assert!(!can_start_hack(2, 3, 15, false));
        assert!(can_start_hack(3, 3, 15, false));
    }
}
const LOG_EDIT_SECONDS: i32 = 30;

//
// ==============================
// RESOLVE IP
// ==============================
//

/// Que hay en una direccion.
///
/// **Devuelve el nodo, no solo el dueno.** Hasta la 0049 la direccion de un jugador era
/// `users.game_ip`, una columna suelta que no pertenecia a ninguna de sus maquinas: con varias
/// por cuenta, «quien es» y «que hay aqui» dejaron de tener la misma respuesta, y esta funcion
/// solo sabia contestar la primera. Ahora se resuelve por la direccion que lleva puesta la
/// maquina, y lo que sale dice ademas cual es.
///
/// `is_npc` sigue aqui y no sobra todavia: de el cuelgan las tablas del dueno. Lo que cambia es
/// que ya no es lo unico que se sabe del otro lado.
pub async fn resolve_ip(
    pool: &PgPool,
    ip: i64,
    viewer: i64,
) -> Result<Remote, ApiError> {
    //
    // IMPORTANTE:
    // NPC tiene prioridad sobre player.
    //

    if let Some(npc) =
        repo::find_npc_by_ip(pool, ip, viewer).await?
    {
        return Ok(Remote {
            is_npc: true,
            id: npc.id,
            name: npc.name,
            npc_type: npc.npc_type,
            down: npc.down,
            web: npc.web,
            server_id: npc.server_id,
        });
    }

    if let Some(player) = repo::find_player_by_ip(pool, ip).await? {
        return Ok(Remote {
            is_npc: false,
            id: player.id,
            name: player.login,
            npc_type: 0,
            down: player.down,
            web: player.web,
            server_id: player.server_id,
        });
    }

    // Y lo ultimo: una direccion de red interna. No esta en Internet, asi que conocerla no
    // basta -- hace falta estar dentro, con acceso vivo en una maquina cableada a ella. Para
    // quien no lo tenga esto es un 404, y esa es justo la gracia: una maquina sin tarjeta de
    // red no existe para el mundo hasta que alguien entra por otra puerta.
    let inside = repo::find_machine_on_lan(pool, ip, viewer)
        .await?
        .ok_or(ApiError::NotFound)?;

    Ok(Remote {
        is_npc: inside.is_npc,
        id: inside.owner_id,
        name: inside.name,
        npc_type: 0,
        down: inside.down,
        web: inside.web,
        server_id: Some(inside.server_id),
    })
}

/// Traduce las concesiones de la base a la lista de verbos que el navegador va a obedecer.
///
/// Un `*` abre la familia entera; un verbo suelto, solo ese. Se resuelve aqui, contra el
/// catalogo, para que la base no tenga que saberse los nombres de las ordenes.
async fn allowed_actions(
    pool: &PgPool,
    npc_id: i64,
) -> Result<Vec<String>, ApiError> {
    use webscript::catalog::{self, Family};

    let mut allowed: Vec<String> = Vec::new();

    for (family, action) in repo::granted_actions(pool, npc_id).await? {
        let family = match family.as_str() {
            "integrated" => Family::Integrated,
            _ => Family::Normal,
        };

        if action == "*" {
            allowed.extend(catalog::of_family(family).into_iter().map(str::to_owned));
        } else if catalog::find(&action).is_some_and(|spec| spec.family == family) {
            allowed.push(action);
        }
    }

    allowed.sort();
    allowed.dedup();

    Ok(allowed)
}

//
// ==============================
// ACCESS
// ==============================
//

pub async fn require_access(
    pool: &PgPool,
    user_id: i64,
    ip: i64,
) -> Result<(), ApiError> {
    let access =
        repo::has_access(
            pool,
            user_id,
            ip,
        )
        .await?;

    if !access {
        return Err(
            ApiError::BadRequest(
                "NOT_LOGGED_IN".into(),
            ),
        );
    }

    Ok(())
}

pub async fn permissions(pool: &PgPool, user: i64, ip: i64) -> Result<Vec<String>, ApiError> {
    Ok(sqlx::query_scalar::<_, serde_json::Value>(
        "SELECT permissions FROM machine_accesses WHERE attacker_id=$1 AND target_ip=$2 AND status='active' AND (expires_at IS NULL OR expires_at>now())")
        .bind(user).bind(ip).fetch_optional(pool).await?
        .and_then(|value| serde_json::from_value(value).ok()).unwrap_or_default())
}

pub async fn require_permission(pool: &PgPool, user: i64, ip: i64, permission: &str) -> Result<(), ApiError> {
    if !permissions(pool, user, ip).await?.iter().any(|p| p == permission) {
        return Err(ApiError::BadRequest("INSUFFICIENT_PERMISSION".into()));
    }
    Ok(())
}

/// Puertas de los servidores simulados. La IP conserva la frontera entre nodos y prácticas.
pub async fn service_intrusion(pool: &PgPool, user: i64, input: super::dtos::ServiceIntrusionRequest) -> Result<crate::software_system::dtos::OperationStarted, ApiError> {
    crate::onboarding::services::guard(pool, user, "red.hackear").await?;
    if !matches!(input.capability.as_str(), "web_exploit" | "ftp_exploit" | "ssh_exploit" | "cracker") {
        return Err(ApiError::BadRequest("INVALID_SERVICE_CAPABILITY".into()));
    }
    let target = resolve_ip(pool, input.ip, user).await?;
    if target.down { return Err(ApiError::BadRequest("SERVER_DOWN".into())); }
    if !target.is_npc && target.id == user { return Err(ApiError::BadRequest("Elige una máquina de práctica u otro jugador".into())); }
    let level = crate::user::repo::effective_certification_level(pool, user).await?;
    let cracker = input.capability == "cracker";
    let own_practice = target.is_npc && repo::is_own_practice(pool, user, target.id).await?;
    if !can_start_hack(level, if cracker {2} else {3}, if cracker {11} else {90}, own_practice) {
        return Err(ApiError::BadRequest("NO_CERTIFICATION".into()));
    }
    if !cracker && !repo::was_scanned(pool, user, input.ip).await? {
        return Err(ApiError::BadRequest("SCAN_REQUIRED: escanea los puertos antes de explotar un servicio".into()));
    }
    let server = target.server_id.ok_or(ApiError::NotFound)?;
    crate::software_system::services::service_intrusion(pool, user,
        crate::software_system::dtos::StartOperationRequest { capability: input.capability, target_user_id: target.id, software_id: input.software_id }, server, target.is_npc).await
}

//
// ==============================
// TARGETS
// ==============================
//

pub async fn internet_targets(
    pool: &PgPool,
    viewer: i64,
) -> Result<Vec<InternetTargetResponse>, ApiError> {
    Ok(
        repo::targets(pool, viewer)
            .await?
            .into_iter()
            .map(InternetTargetResponse::from)
            .collect(),
    )
}

//
// ==============================
// NAVIGATE
// ==============================
//

pub async fn internet_navigate(
    pool: &PgPool,
    user_id: i64,
    ip: i64,
) -> Result<InternetNavigateResponse, ApiError> {
    let mut target = resolve_ip(
        pool,
        ip,
        user_id,
    )
    .await?;

    if let Some(server) = target.server_id {
        if let Some(content) = sqlx::query_scalar::<_, String>("SELECT content FROM node_web_content WHERE server_id=$1")
            .bind(server).fetch_optional(pool).await? {
            target.web = Some(content);
        }
    }

    repo::save_history(
        pool,
        user_id,
        ip,
    )
    .await?;

    let permissions = permissions(pool, user_id, ip).await?;
    let access = !permissions.is_empty();

    let access_as = if access {
        repo::access_login(pool, user_id, ip).await?
    } else {
        None
    };

    let bookmarked = repo::is_bookmarked(
        pool,
        user_id,
        ip,
    )
    .await?;

    let scanned = repo::was_scanned(
        pool,
        user_id,
        ip,
    )
    .await?;

    let analysis = repo::analysis(
        pool,
        user_id,
        ip,
    )
    .await?
    .map(HardwareAnalysisResponse::from);

    Ok(InternetNavigateResponse {
        ip,

        kind: if target.is_npc {
            "npc"
        } else {
            "player"
        },

        id: target.id,
        name: target.name,
        npc_type: target.npc_type,
        down: target.down,
        access,
        access_as,
        permissions,
        server_id: target.server_id,
        bookmarked,
        scanned,
        analysis,

        // El sitio se lee aqui y no en el cliente para que el lenguaje tenga un solo
        // interprete. Cuando no declara nada, no se manda nada: el navegador pinta el HTML
        // tal cual y se ahorra el rodeo.
        site: target.web.as_deref().map(webscript::parse).filter(|site| !site.is_plain()),

        // Solo tiene sentido preguntarlo de un NPC: un jugador no es un banco.
        bank: if target.is_npc {
            repo::bank_of_npc(pool, target.id)
                .await?
                .map(BankRef::from)
        } else {
            None
        },

        allowed: if target.is_npc {
            allowed_actions(pool, target.id).await?
        } else {
            // La pagina de un jugador solo puede hacer lo que puede hacer cualquiera.
            webscript::catalog::of_family(webscript::catalog::Family::Normal)
                .into_iter()
                .map(str::to_owned)
                .collect()
        },

        web: target.web,
    })
}

//
// ==============================
// LOGIN
// ==============================
//

pub async fn internet_login(
    pool: &PgPool,
    user_id: i64,
    input: InternetLoginRequest,
) -> Result<InternetLoginResponse, ApiError> {
    crate::onboarding::services::guard(pool, user_id, "red.entrar").await?;

    if input.user.trim() != "root" {
        return Err(
            ApiError::BadRequest(
                "UNKNOWN_USER".into(),
            ),
        );
    }

    let target =
        resolve_ip(pool, input.ip, user_id).await?;

    if target.down {
        return Err(
            ApiError::BadRequest(
                "SERVER_DOWN".into(),
            ),
        );
    }

    // **La llave es de la maquina, no del dueno.** Antes se comprobaba la del propietario, asi
    // que acertarla en una direccion abria todas las suyas y tener direcciones separadas era
    // decorativo. Ahora cada equipo tiene la suya y entrar en uno es entrar en uno.
    let expected = match target.server_id {
        Some(machine) => repo::machine_password(pool, machine).await?,
        None => None,
    };

    let Some(expected) = expected else {
        return Err(ApiError::BadRequest("SERVER_DOWN".into()));
    };

    if input.password != expected {
        return Err(
            ApiError::BadRequest(
                "BAD_PASSWORD".into(),
            ),
        );
    }

    repo::grant_access(
        pool,
        user_id,
        input.ip,
    )
    .await?;

    sqlx::query("INSERT INTO machine_accesses(attacker_id,target_user_id,target_is_npc,target_ip,identity_name,origin,privilege,scope,permissions)
        SELECT $1,$2,$3,$4,'root','login','root','root',granted_permissions FROM software_capabilities WHERE code='cracker'
        ON CONFLICT(attacker_id,target_ip) DO UPDATE SET identity_name='root',privilege='root',scope='root',permissions=EXCLUDED.permissions,status='active',expires_at=NULL,obtained_at=now()")
        .bind(user_id).bind(target.id).bind(target.is_npc).bind(input.ip).execute(pool).await?;

    Ok(InternetLoginResponse {
        access: true,
        name: target.name,
    })
}

//
// ==============================
// HACK
// ==============================
//

pub async fn internet_hack(
    pool: &PgPool,
    user_id: i64,
    input: InternetHackRequest,
) -> Result<InternetHackResponse, ApiError> {
    crate::onboarding::services::guard(pool, user_id, "red.hackear").await?;
    let (required, action) = match input.kind.as_str() {
        "bf" => (2,11),
        "scan" => (3,15),
        _ => return Err(ApiError::BadRequest("unsupported hack type".into())),
    };
    let level = crate::user::repo::effective_certification_level(pool, user_id).await?;
    let target = resolve_ip(pool, input.ip, user_id).await?;
    let practice = target.is_npc && repo::is_own_practice(pool,user_id,target.id).await?;
    if !can_start_hack(level,required,action,practice) {
        return Err(ApiError::BadRequest("NO_CERTIFICATION".into()));
    }
    if target.down { return Err(ApiError::BadRequest("SERVER_DOWN".into())); }
    let server = target.server_id.ok_or(ApiError::NotFound)?;
    let operation = if input.kind == "scan" {
        crate::software_system::services::port_scan(pool,user_id,target.id,server,target.is_npc,input.software_id).await?
    } else {
        crate::software_system::services::brute_force(pool,user_id,target.id,server,target.is_npc).await?
    };
    Ok(InternetHackResponse {
        process_id: operation.process_id,action,seconds: operation.seconds,
        attacker_version:None,defender_version:None,
        estimated_chance:Some(operation.estimated_chance),attacker_stats:Some(operation.attacker),
    })
}

//
// ==============================
// REMOTE LOG
// ==============================
//

pub async fn remote_log(
    pool: &PgPool,
    user_id: i64,
    ip: i64,
) -> Result<RemoteLogResponse, ApiError> {
    let target =
        resolve_ip(pool, ip, user_id).await?;

    require_permission(
        pool,
        user_id,
        ip,
        "read_logs",
    )
    .await?;

    let text =
        repo::remote_log(
            pool,
            target.id,
            target.is_npc,
        )
        .await?
        .unwrap_or_default();

    Ok(RemoteLogResponse {
        ip,
        text,
    })
}

pub async fn edit_remote_log(
    pool: &PgPool,
    user_id: i64,
    ip: i64,
    input: LogEditRequest,
) -> Result<LogEditStartedResponse, ApiError> {
    let target =
        resolve_ip(pool, ip, user_id).await?;

    require_permission(
        pool,
        user_id,
        ip,
        "modify_logs",
    )
    .await?;

    let info = if target.is_npc {
        "npc"
    } else {
        "player"
    };

    let mut tx = pool.begin().await?;
    let seconds = crate::process::services::duration_for(LOG_EDIT_SECONDS);

    let process_id =
        repo::create_log_edit_process(
            &mut tx,
            user_id,
            target.id,
            target.server_id,
            info,
            seconds,
        )
        .await?;

    repo::create_log_edit(
        &mut tx,
        target.id,
        target.is_npc,
        user_id,
        &input.text,
        process_id,
    )
    .await?;

    tx.commit().await?;

    Ok(LogEditStartedResponse {
        process_id,
        seconds,
    })
}

//
// ==============================
// ANALYZE
// ==============================
//

pub async fn internet_analyze(
    pool: &PgPool,
    user_id: i64,
    input: IpRequest,
) -> Result<AnalyzeStartedResponse, ApiError> {
    crate::onboarding::services::guard(pool, user_id, "red.analizar").await?;

    let target =
        resolve_ip(pool, input.ip, user_id).await?;

    let analyzer =
        repo::best_running_analyzer(
            pool,
            user_id,
        )
        .await?
        .ok_or_else(|| {
            ApiError::BadRequest(
                "NO_SOFT".into(),
            )
        })?;

    let duplicate =
        repo::active_analyze_exists(
            pool,
            user_id,
            target.id,
        )
        .await?;

    if duplicate {
        return Err(
            ApiError::BadRequest(
                "identical process already active".into(),
            ),
        );
    }

    let info = if target.is_npc {
        "npc"
    } else {
        "player"
    };

    let seconds = crate::process::services::duration_for(ANALYZE_SECONDS);
    let process_id =
        repo::create_analyze_process(
            pool,
            user_id,
            target.id,
            target.server_id,
            analyzer.id,
            info,
            seconds,
        )
        .await?;

    Ok(AnalyzeStartedResponse {
        process_id,
        seconds,
        analyzer_version: analyzer.soft_version,
    })
}

//
// ==============================
// BOOKMARKS
// ==============================
//

pub async fn internet_bookmarks(
    pool: &PgPool,
    user_id: i64,
) -> Result<Vec<BookmarkResponse>, ApiError> {
    Ok(
        repo::bookmarks(
            pool,
            user_id,
        )
        .await?
        .into_iter()
        .map(BookmarkResponse::from)
        .collect(),
    )
}

pub async fn add_bookmark(
    pool: &PgPool,
    user_id: i64,
    input: IpRequest,
) -> Result<BookmarkCreatedResponse, ApiError> {
    //
    // Validate target exists.
    //

    resolve_ip(
        pool,
        input.ip,
        user_id,
    )
    .await?;

    repo::add_bookmark(
        pool,
        user_id,
        input.ip,
    )
    .await?;

    Ok(BookmarkCreatedResponse {
        ip: input.ip,
    })
}

pub async fn remove_bookmark(
    pool: &PgPool,
    user_id: i64,
    ip: i64,
) -> Result<(), ApiError> {
    let deleted =
        repo::remove_bookmark(
            pool,
            user_id,
            ip,
        )
        .await?;

    if !deleted {
        return Err(ApiError::NotFound);
    }

    Ok(())
}

//
// ==============================
// HISTORY
// ==============================
//

pub async fn internet_history(
    pool: &PgPool,
    user_id: i64,
) -> Result<Vec<InternetHistoryResponse>, ApiError> {
    Ok(
        repo::history(
            pool,
            user_id,
        )
        .await?
        .into_iter()
        .map(InternetHistoryResponse::from)
        .collect(),
    )
}
//
// ==============================
// SITIOS
// ==============================
//

/// Todas las paginas, con su contenido.
///
/// Va entero y no por trozos porque el panel deja saltar de una a otra editandolas, y
/// pedir cada una al hacer clic convertiria eso en una espera por sitio. Son decenas de
/// paginas de un par de kilobytes; el dia que sean miles, se pagina.
pub async fn all_sites(
    pool: &PgPool,
    ctx: crate::auth::Ctx,
) -> Result<Vec<crate::internet::dtos::SiteResponse>, ApiError> {
    ctx.require_admin()?;

    let sites = repo::all_sites(pool)
        .await?
        .into_iter()
        .map(crate::internet::dtos::SiteResponse::from)
        .collect();

    Ok(sites)
}

/// Cambiar la pagina de cualquiera. Pasa por la misma regla que el perfil de un jugador:
/// ser administrador da permiso para editar sitios ajenos, no para meter un guion.
pub async fn set_site(
    pool: &PgPool,
    ctx: crate::auth::Ctx,
    kind: &str,
    id: i64,
    web: &str,
) -> Result<(), ApiError> {
    ctx.require_admin()?;

    super::sanitize::check(web)?;

    let changed = match kind {
        "npc" => repo::set_npc_site(pool, id, web).await?,
        "player" => repo::set_player_site(pool, id, web).await?,
        _ => return Err(ApiError::BadRequest("kind must be npc or player".into())),
    };

    if changed == 0 {
        return Err(ApiError::BadRequest("unknown site".into()));
    }

    Ok(())
}

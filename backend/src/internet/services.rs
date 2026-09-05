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

const HACK_SECONDS: i32 = 60;
const ANALYZE_SECONDS: i32 = 60;
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
) -> Result<Remote, ApiError> {
    //
    // IMPORTANTE:
    // NPC tiene prioridad sobre player.
    //

    if let Some(npc) =
        repo::find_npc_by_ip(pool, ip).await?
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

    let player =
        repo::find_player_by_ip(pool, ip)
            .await?
            .ok_or(ApiError::NotFound)?;

    Ok(Remote {
        is_npc: false,
        id: player.id,
        name: player.login,
        npc_type: 0,
        down: false,
        web: player.web,
        server_id: player.server_id,
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

//
// ==============================
// TARGETS
// ==============================
//

pub async fn internet_targets(
    pool: &PgPool,
) -> Result<Vec<InternetTargetResponse>, ApiError> {
    Ok(
        repo::targets(pool)
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
    let target = resolve_ip(
        pool,
        ip,
    )
    .await?;

    repo::save_history(
        pool,
        user_id,
        ip,
    )
    .await?;

    let access = repo::has_access(
        pool,
        user_id,
        ip,
    )
    .await?;

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
    if input.user.trim() != "root" {
        return Err(
            ApiError::BadRequest(
                "UNKNOWN_USER".into(),
            ),
        );
    }

    let target =
        resolve_ip(pool, input.ip).await?;

    if target.down {
        return Err(
            ApiError::BadRequest(
                "SERVER_DOWN".into(),
            ),
        );
    }

    let expected = if target.is_npc {
        repo::npc_password(
            pool,
            target.id,
        )
        .await?
    } else {
        repo::player_password(
            pool,
            target.id,
        )
        .await?
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
    let (
        attacker_type,
        defender_type,
        certification_needed,
        action,
    ): (i16, i16, i16, i16) =
        match input.kind.as_str() {
            "bf" => (1, 2, 2, 11),
            "scan" => (3, 4, 3, 15),

            _ => {
                return Err(
                    ApiError::BadRequest(
                        "unsupported hack type".into(),
                    ),
                );
            }
        };

    let certification =
        repo::certification_level(
            pool,
            user_id,
        )
        .await?
        .unwrap_or(0);

    if certification < certification_needed {
        return Err(
            ApiError::BadRequest(
                "NO_CERTIFICATION".into(),
            ),
        );
    }

    let target =
        resolve_ip(pool, input.ip).await?;

    if target.down {
        return Err(
            ApiError::BadRequest(
                "SERVER_DOWN".into(),
            ),
        );
    }

    let attacker =
        repo::best_local_software_version(
            pool,
            user_id,
            attacker_type,
        )
        .await?
        .ok_or_else(|| {
            ApiError::BadRequest(
                "NO_SOFT".into(),
            )
        })?;

    let defender =
        repo::best_remote_software_version(
            pool,
            target.id,
            defender_type,
            target.is_npc,
        )
        .await?;

    if attacker < defender {
        return Err(
            ApiError::BadRequest(
                "BAD_CRACKER".into(),
            ),
        );
    }

    let duplicate =
        repo::active_hack_exists(
            pool,
            user_id,
            action,
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

    let seconds = crate::process::services::duration_for(HACK_SECONDS);
    let process_id =
        repo::create_hack_process(
            pool,
            user_id,
            target.id,
            action,
            info,
            seconds,
        )
        .await?;

    Ok(InternetHackResponse {
        process_id,
        action,
        seconds,
        attacker_version: attacker,
        defender_version: defender,
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
        resolve_ip(pool, ip).await?;

    require_access(
        pool,
        user_id,
        ip,
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
        resolve_ip(pool, ip).await?;

    require_access(
        pool,
        user_id,
        ip,
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
    let target =
        resolve_ip(pool, input.ip).await?;

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

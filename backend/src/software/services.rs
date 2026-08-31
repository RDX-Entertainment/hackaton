use sqlx::PgPool;

use crate::error::ApiError;

use crate::Generic::models::CatalogEntry;
use crate::legacy::SOFTWARE_TYPES;

use super::{
    dtos::{
        DeleteSoftwareStartedResponse,
        AdminSoftwareRequest,
        MoveSoftwareRequest,
        NewSoftwareRequest,
        RemoteSoftwareResponse,
        ResearchHistoryResponse,
        ResearchStartRequest,
        ResearchStartedResponse,
        SoftwareCreatedResponse,
        SoftwareResponse,
        TransferRequest,
        TransferStartedResponse,
    },
    repo,
};

const TRANSFER_SECONDS: i32 = 30;
const RESEARCH_SECONDS: i32 = 60;

//
// ==============================
// SOFTWARE TYPES
// ==============================
//

/// Los nombres de PC::getSoftwareName. El frontend recibe soft_type como un numero en
/// cada payload de software, asi que necesita esta tabla para no pintar ids opacos.
pub fn software_type_catalog() -> Vec<CatalogEntry> {
    SOFTWARE_TYPES
        .iter()
        .map(|(name, id)| CatalogEntry {
            name,
            id: *id,
        })
        .collect()
}

//
// ==============================
// LOCAL SOFTWARE
// ==============================
//

pub async fn software(
    pool: &PgPool,
    user_id: i64,
) -> Result<Vec<SoftwareResponse>, ApiError> {
    let result = repo::find_user_software(
        pool,
        user_id,
    )
    .await?
    .into_iter()
    .map(SoftwareResponse::from)
    .collect();

    Ok(result)
}

pub async fn create_software(
    pool: &PgPool,
    user_id: i64,
    input: NewSoftwareRequest,
) -> Result<SoftwareCreatedResponse, ApiError> {
    let name = input.name.trim();

    if name.is_empty() || name.len() > 25 {
        return Err(
            ApiError::BadRequest(
                "invalid software name".into(),
            ),
        );
    }

    let valid_type = SOFTWARE_TYPES
        .iter()
        .any(|(_, id)| {
            *id as i16 == input.soft_type
        });

    if !valid_type {
        return Err(
            ApiError::BadRequest(
                "unknown software type".into(),
            ),
        );
    }

    if input.size <= 0 || input.ram < 0 {
        return Err(
            ApiError::BadRequest(
                "invalid size".into(),
            ),
        );
    }

    // El equipo tiene que ser suyo. `machine_hdd` filtra por dueno, asi que un `None` aqui
    // significa «esa maquina no existe o no es tuya» -- las dos merecen la misma respuesta,
    // porque distinguirlas le diria a cualquiera que maquinas hay.
    let Some(total) = repo::machine_hdd(pool, input.server_id, user_id).await? else {
        return Err(ApiError::BadRequest("unknown machine".into()));
    };

    let total = total as i64;
    let used = repo::used_hdd(pool, input.server_id).await?.unwrap_or(0);

    if total < used + input.size as i64 {
        return Err(ApiError::BadRequest(format!(
            "NO_DISK_SPACE: en ese equipo caben {total} MB y ya hay {used} MB"
        )));
    }

    let id = repo::create_software(
        pool,
        user_id,
        input.server_id,
        name,
        input.size,
        input.ram,
        input.soft_type,
    )
    .await?;

    Ok(SoftwareCreatedResponse {
        id,
        name: name.to_owned(),
        version: 1,
    })
}

//
// ==============================
// REMOTE SOFTWARE
// ==============================
//

pub async fn remote_software(
    pool: &PgPool,
    owner_id: i64,
    owner_is_npc: bool,
) -> Result<Vec<RemoteSoftwareResponse>, ApiError> {
    let result =
        repo::find_remote_software(
            pool,
            owner_id,
            owner_is_npc,
        )
        .await?
        .into_iter()
        .map(RemoteSoftwareResponse::from)
        .collect();

    Ok(result)
}

//
// ==============================
// TRANSFER
// ==============================
//

fn transferable(
    soft_type: i16,
) -> bool {
    soft_type != 19
        && soft_type != 31
        && soft_type < 90
}

pub async fn transfer_software(
    pool: &PgPool,
    user_id: i64,
    target_id: i64,
    target_is_npc: bool,
    input: TransferRequest,
) -> Result<TransferStartedResponse, ApiError> {
    let action: i16 =
        match input.direction.as_str() {
            "download" => 1,
            "upload" => 2,

            _ => {
                return Err(
                    ApiError::BadRequest(
                        "unsupported transfer".into(),
                    ),
                );
            }
        };

    //
    // Download:
    // el archivo pertenece a la máquina remota.
    //
    // Upload:
    // el archivo pertenece al jugador.
    //

    let (
        owner_id,
        owner_is_npc,
    ) = if action == 1 {
        (
            target_id,
            target_is_npc,
        )
    } else {
        (
            user_id,
            false,
        )
    };

    let software =
        repo::find_transfer_software(
            pool,
            input.software_id,
            owner_id,
            owner_is_npc,
        )
        .await?
        .ok_or(ApiError::NotFound)?;

    if !transferable(
        software.soft_type,
    ) {
        return Err(
            ApiError::BadRequest(
                "CANNOT_TRANSFER".into(),
            ),
        );
    }

    let duplicate =
        repo::active_transfer_exists(
            pool,
            user_id,
            action,
            target_id,
            input.software_id,
        )
        .await?;

    if duplicate {
        return Err(
            ApiError::BadRequest(
                "identical process already active".into(),
            ),
        );
    }

    let info = if target_is_npc {
        "npc"
    } else {
        "player"
    };

    let seconds = crate::process::services::duration_for(TRANSFER_SECONDS);
    let process_id =
        repo::create_transfer_process(
            pool,
            user_id,
            target_id,
            action,
            input.software_id,
            info,
            seconds,
        )
        .await?;

    Ok(
        TransferStartedResponse {
            process_id,
            action,
            seconds,
        },
    )
}

//
// ==============================
// DELETE REMOTE SOFTWARE
// ==============================
//

pub async fn delete_remote_software(
    pool: &PgPool,
    user_id: i64,
    target_id: i64,
    target_is_npc: bool,
    software_id: i64,
) -> Result<DeleteSoftwareStartedResponse, ApiError> {
    let exists =
        repo::remote_software_exists(
            pool,
            software_id,
            target_id,
            target_is_npc,
        )
        .await?;

    if !exists {
        return Err(ApiError::NotFound);
    }

    let info = if target_is_npc {
        "npc"
    } else {
        "player"
    };

    let seconds = crate::process::services::duration_for(TRANSFER_SECONDS);
    let process_id =
        repo::create_delete_process(
            pool,
            user_id,
            target_id,
            software_id,
            info,
            seconds,
        )
        .await?;

    Ok(
        DeleteSoftwareStartedResponse {
            process_id,
            seconds,
        },
    )
}

//
// ==============================
// RESEARCH
// ==============================
//

pub async fn research(
    pool: &PgPool,
    user_id: i64,
) -> Result<Vec<ResearchHistoryResponse>, ApiError> {
    let result =
        repo::research_history(
            pool,
            user_id,
        )
        .await?
        .into_iter()
        .map(ResearchHistoryResponse::from)
        .collect();

    Ok(result)
}

pub async fn start_research(
    pool: &PgPool,
    user_id: i64,
    input: ResearchStartRequest,
) -> Result<ResearchStartedResponse, ApiError> {
    let exists =
        repo::local_software_exists(
            pool,
            user_id,
            input.software_id,
        )
        .await?;

    if !exists {
        return Err(ApiError::NotFound);
    }

    let duplicate =
        repo::active_research_exists(
            pool,
            user_id,
            input.software_id,
        )
        .await?;

    if duplicate {
        return Err(
            ApiError::BadRequest(
                "identical process already active".into(),
            ),
        );
    }

    let seconds = crate::process::services::duration_for(RESEARCH_SECONDS);
    let process_id =
        repo::create_research_process(
            pool,
            user_id,
            input.software_id,
            seconds,
        )
        .await?;

    Ok(
        ResearchStartedResponse {
            process_id,
            software_id: input.software_id,
            seconds,
        },
    )
}

fn validate_admin_software(input: &AdminSoftwareRequest) -> Result<&str, ApiError> {
    let name = input.name.trim();
    if name.is_empty() || name.len() > 25 || input.version < 1 || input.size <= 0 || input.ram < 0 {
        return Err(ApiError::BadRequest("invalid software values".into()));
    }
    if !SOFTWARE_TYPES.iter().any(|(_, id)| *id as i16 == input.soft_type) {
        return Err(ApiError::BadRequest("unknown software type".into()));
    }
    Ok(name)
}

pub async fn admin_software(pool: &PgPool, user_id: i64) -> Result<Vec<SoftwareResponse>, ApiError> {
    if !repo::user_exists(pool, user_id).await? { return Err(ApiError::NotFound); }
    software(pool, user_id).await
}

pub async fn admin_create_software(pool: &PgPool, user_id: i64, input: AdminSoftwareRequest) -> Result<i64, ApiError> {
    let name = validate_admin_software(&input)?;
    if !repo::user_exists(pool, user_id).await? { return Err(ApiError::NotFound); }
    let id = repo::create_admin_software(pool, user_id, name, input.version, input.size, input.ram, input.soft_type).await?;
    repo::set_admin_software_running(pool, user_id, id, input.running).await?;
    Ok(id)
}

pub async fn admin_update_software(pool: &PgPool, user_id: i64, software_id: i64, input: AdminSoftwareRequest) -> Result<(), ApiError> {
    let name = validate_admin_software(&input)?;
    if !repo::update_admin_software(pool, user_id, software_id, name, input.version, input.size, input.ram, input.soft_type).await? {
        return Err(ApiError::NotFound);
    }
    repo::set_admin_software_running(pool, user_id, software_id, input.running).await?;
    Ok(())
}

pub async fn admin_delete_software(pool: &PgPool, user_id: i64, software_id: i64) -> Result<(), ApiError> {
    if !repo::delete_admin_software(pool, user_id, software_id).await? { return Err(ApiError::NotFound); }
    Ok(())
}

pub async fn admin_set_software_running(pool: &PgPool, user_id: i64, software_id: i64, running: bool) -> Result<(), ApiError> {
    if !repo::set_admin_software_running(pool, user_id, software_id, running).await? { return Err(ApiError::NotFound); }
    Ok(())
}

//
// ==============================
// MOVER ENTRE MAQUINAS
// ==============================
//

/// Llevar un fichero de un equipo a otro.
///
/// Desde que el software vive en un disco concreto, esto deja de ser un lujo: sin forma de
/// moverlo, lo que caiga en el equipo pequeño se queda ahi para siempre y la unica salida es
/// borrarlo.
///
/// ## Hace falta un camino
///
/// Un fichero no salta por el aire entre dos maquinas: o comparten cable, o las dos salen a
/// Internet y se lo pasan por ahi. Una maquina sin tarjeta y sin cable esta aislada de
/// verdad, y eso es lo que hace que aislar algo signifique algo -- lo que metes ahi, ahi se
/// queda hasta que le tiendas un cable.
///
/// Exigir solo cable habria sido mas puro y habria bloqueado el juego: una placa domestica
/// tiene una boca, y si la ocupa la salida a Internet no queda ninguna para cablear.
pub async fn move_software(
    pool: &PgPool,
    user_id: i64,
    software: i64,
    input: MoveSoftwareRequest,
) -> Result<(), ApiError> {
    let Some((origen, size, running)) =
        repo::find_owned_software(pool, software, user_id).await?
    else {
        return Err(ApiError::BadRequest("unknown software".into()));
    };

    let destino = input.server_id;

    if origen == destino {
        return Err(ApiError::BadRequest(
            "SAME_MACHINE: ese fichero ya esta en ese equipo".into(),
        ));
    }

    // Cargado en memoria no se mueve: primero se descarga. Mover algo que esta corriendo
    // seria sacarle el disco de debajo a un programa en marcha.
    if running {
        return Err(ApiError::BadRequest(
            "SOFTWARE_RUNNING: descargalo de memoria antes de moverlo".into(),
        ));
    }

    // `machine_hdd` filtra por dueno: un `None` es «no existe o no es tuya», y las dos
    // merecen la misma respuesta.
    let Some(disco) = repo::machine_hdd(pool, destino, user_id).await? else {
        return Err(ApiError::BadRequest("unknown machine".into()));
    };

    let ocupado = repo::used_hdd(pool, destino).await?.unwrap_or(0);
    let disco = disco as i64;

    if disco < ocupado + size as i64 {
        return Err(ApiError::BadRequest(format!(
            "NO_DISK_SPACE: en ese equipo caben {disco} MB y ya hay {ocupado} MB"
        )));
    }

    let por_cable = repo::are_linked(pool, origen, destino).await?;

    let por_internet = repo::has_public_ip(pool, origen).await?
        && repo::has_public_ip(pool, destino).await?;

    if !por_cable && !por_internet {
        return Err(ApiError::BadRequest(
            "NO_ROUTE: esos dos equipos no se ven; tiendeles un cable o dales salida a \
             Internet a los dos"
                .into(),
        ));
    }

    repo::set_software_machine(pool, software, destino).await?;

    Ok(())
}

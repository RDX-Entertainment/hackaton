use sqlx::PgPool;

use crate::error::ApiError;

use crate::Generic::models::CatalogEntry;
use crate::legacy::SOFTWARE_TYPES;

use super::{
    dtos::{
        DeleteSoftwareStartedResponse,
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

    let total = repo::total_hdd(
        pool,
        user_id,
    )
    .await?
    .unwrap_or(0.0) as i64;

    let used = repo::used_hdd(
        pool,
        user_id,
    )
    .await?
    .unwrap_or(0);

    if total < used + input.size as i64 {
        return Err(
            ApiError::BadRequest(
                "NO_DISK_SPACE".into(),
            ),
        );
    }

    let id = repo::create_software(
        pool,
        user_id,
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

    let process_id =
        repo::create_transfer_process(
            pool,
            user_id,
            target_id,
            action,
            input.software_id,
            info,
            TRANSFER_SECONDS,
        )
        .await?;

    Ok(
        TransferStartedResponse {
            process_id,
            action,
            seconds: TRANSFER_SECONDS,
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

    let process_id =
        repo::create_delete_process(
            pool,
            user_id,
            target_id,
            software_id,
            info,
            TRANSFER_SECONDS,
        )
        .await?;

    Ok(
        DeleteSoftwareStartedResponse {
            process_id,
            seconds: TRANSFER_SECONDS,
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

    let process_id =
        repo::create_research_process(
            pool,
            user_id,
            input.software_id,
            RESEARCH_SECONDS,
        )
        .await?;

    Ok(
        ResearchStartedResponse {
            process_id,
            software_id: input.software_id,
            seconds: RESEARCH_SECONDS,
        },
    )
}
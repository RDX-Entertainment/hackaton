use sqlx::PgPool;

use crate::error::ApiError;

use super::{
    dtos::{
        MissionCommandResponse,
        MissionResponse,
    },
    repo,
};

//
// ==============================
// BOARD
// ==============================
//

pub async fn missions(
    pool: &PgPool,
    user_id: i64,
) -> Result<Vec<MissionResponse>, ApiError> {
    let missions =
        repo::find_visible(pool, user_id)
            .await?
            .into_iter()
            .map(MissionResponse::from)
            .collect();

    Ok(missions)
}

//
// ==============================
// COMMANDS
// ==============================
//

pub async fn command(
    pool: &PgPool,
    user_id: i64,
    mission_id: i64,
    action: String,
) -> Result<MissionCommandResponse, ApiError> {
    let affected = match action.as_str() {
        "accept" => {
            crate::onboarding::services::guard(pool, user_id, "encargo.aceptar").await?;

            accept(pool, user_id, mission_id)
                .await?
        }
        "abort" => {
            repo::abort(
                pool,
                mission_id,
                user_id,
            )
            .await?
        }
        _ => {
            return Err(ApiError::BadRequest(
                "unsupported mission action"
                    .into(),
            ))
        }
    };

    if affected == 0 {
        return Err(ApiError::BadRequest(
            "mission action not allowed".into(),
        ));
    }

    Ok(MissionCommandResponse {
        id: mission_id,
        action,
    })
}

//
// ==============================
// ACCEPT RULES
// ==============================
//

async fn accept(
    pool: &PgPool,
    user_id: i64,
    mission_id: i64,
) -> Result<u64, ApiError> {
    // Una mision activa por jugador, como en el legado.
    let active =
        repo::count_active(pool, user_id).await?;

    if active != 0 {
        return Err(ApiError::BadRequest(
            "you already are in a mission".into(),
        ));
    }

    let level = repo::find_open_level(
        pool,
        mission_id,
    )
    .await?
    .ok_or(ApiError::NotFound)?;

    let player_level = repo::certification_level(pool, user_id).await?;

    if level != player_level {
        return Err(ApiError::BadRequest(
            "mission level does not match your certification"
                .into(),
        ));
    }

    Ok(repo::accept(pool, mission_id, user_id)
        .await?)
}

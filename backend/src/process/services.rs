use std::env;

use sqlx::PgPool;

use crate::error::ApiError;
use crate::legacy::PROCESS_ACTIONS;
use crate::Generic::models::CatalogEntry;

use super::{
    dtos::{
        ProcessPausedResponse,
        ProcessResponse,
        ProcessStartedResponse,
        StartProcessRequest,
    },
    repo,
};

//
// ==============================
// ACTION CATALOGUE
// ==============================
//

/// Los nombres de las acciones heredadas. La cola de React los usa para no mostrar ids
/// desnudos al jugador.
pub fn action_catalog() -> Vec<CatalogEntry> {
    PROCESS_ACTIONS
        .iter()
        .map(|(name, id)| CatalogEntry {
            name,
            id: *id,
        })
        .collect()
}

//
// ==============================
// QUEUE
// ==============================
//

pub async fn processes(
    pool: &PgPool,
    user_id: i64,
) -> Result<Vec<ProcessResponse>, ApiError> {
    let processes =
        repo::find_for_user(pool, user_id)
            .await?
            .into_iter()
            .map(ProcessResponse::from)
            .collect();

    Ok(processes)
}

pub async fn process(
    pool: &PgPool,
    user_id: i64,
    process_id: i64,
) -> Result<ProcessResponse, ApiError> {
    repo::find_one_for_user(pool, process_id, user_id)
        .await?
        .map(ProcessResponse::from)
        .ok_or(ApiError::NotFound)
}

//
// ==============================
// START
// ==============================
//

/// El campo info del legado cabia en treinta caracteres.
const INFO_MAX: usize = 30;

/// Cuanto tarda cada accion. El legado las repartia en tres tramos segun lo costosa que
/// fuera la operacion.
fn duration_of(action: u8) -> i32 {
    let seconds = match action {
        1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 13
        | 14 => 30,
        10 | 11 | 12 | 15 | 16 | 17 | 22 | 23 => {
            60
        }
        18 | 19 | 20 | 24 | 25 | 26 | 27 | 28 => {
            120
        }
        _ => 60,
    };
    duration_for(seconds)
}

/// El entorno de desarrollo puede reducir todas las colas a 3--5 segundos sin cambiar la
/// formula real de cada accion. Produccion simplemente no define esta variable.
pub fn duration_for(seconds: i32) -> i32 {
    env::var("PROCESS_TEST_SECONDS")
        .ok()
        .and_then(|value| value.parse::<i32>().ok())
        .filter(|value| (3..=5).contains(value))
        .unwrap_or(seconds)
}

pub async fn start(
    pool: &PgPool,
    user_id: i64,
    input: StartProcessRequest,
) -> Result<ProcessStartedResponse, ApiError> {
    let known = PROCESS_ACTIONS
        .iter()
        .any(|(_, id)| *id == input.action);

    if !known {
        return Err(ApiError::BadRequest(
            "unsupported process action".into(),
        ));
    }

    let info = input
        .info
        .as_deref()
        .unwrap_or("")
        .trim()
        .to_owned();

    if info.len() > INFO_MAX {
        return Err(ApiError::BadRequest(
            "process info is too long".into(),
        ));
    }

    let software_id =
        input.software_id.unwrap_or(0);

    // Un proceso sobre software ajeno no es asunto de quien lo pide.
    if software_id != 0
        && !repo::owns_software(
            pool,
            software_id,
            user_id,
        )
        .await?
    {
        return Err(ApiError::NotFound);
    }

    let victim_id = input.victim_id.unwrap_or(0);
    let seconds = duration_of(input.action);

    let duplicate = repo::duplicate_exists(
        pool,
        user_id,
        input.action as i16,
        victim_id,
        software_id,
        &info,
    )
    .await?;

    if duplicate {
        return Err(ApiError::BadRequest(
            "identical process already active"
                .into(),
        ));
    }

    let id = repo::insert(
        pool,
        user_id,
        victim_id,
        input.action as i16,
        software_id,
        &info,
        seconds,
    )
    .await?;

    Ok(ProcessStartedResponse {
        id,
        action: input.action,
        seconds,
    })
}

//
// ==============================
// DELETE
// ==============================
//

pub async fn delete(
    pool: &PgPool,
    user_id: i64,
    process_id: i64,
) -> Result<(), ApiError> {
    let affected =
        repo::delete(pool, process_id, user_id)
            .await?;

    if affected == 0 {
        return Err(ApiError::NotFound);
    }

    Ok(())
}

//
// ==============================
// PAUSE
// ==============================
//

/// Pausar guarda lo que le quedaba por delante; reanudar se lo devuelve entero, de modo
/// que una pausa larga no adelanta trabajo.
pub async fn pause(
    pool: &PgPool,
    user_id: i64,
    process_id: i64,
) -> Result<ProcessPausedResponse, ApiError> {
    let mut tx = pool.begin().await?;

    let end = repo::lock_running_end(
        &mut tx,
        process_id,
        user_id,
    )
    .await?
    .ok_or(ApiError::NotFound)?;

    let seconds_left =
        repo::seconds_until(&mut tx, end).await?;

    repo::pause(
        &mut tx,
        process_id,
        user_id,
        seconds_left,
    )
    .await?;

    tx.commit().await?;

    Ok(ProcessPausedResponse {
        id: process_id,
        paused: true,
        seconds_left,
    })
}

//
// ==============================
// RESUME
// ==============================
//

pub async fn resume(
    pool: &PgPool,
    user_id: i64,
    process_id: i64,
) -> Result<ProcessPausedResponse, ApiError> {
    let mut tx = pool.begin().await?;

    let seconds_left =
        repo::lock_paused_seconds(
            &mut tx,
            process_id,
            user_id,
        )
        .await?
        .ok_or(ApiError::NotFound)?;

    repo::resume(
        &mut tx,
        process_id,
        seconds_left,
    )
    .await?;

    tx.commit().await?;

    Ok(ProcessPausedResponse {
        id: process_id,
        paused: false,
        seconds_left,
    })
}

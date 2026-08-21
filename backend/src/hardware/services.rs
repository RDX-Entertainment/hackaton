use sqlx::PgPool;

use crate::error::ApiError;
use crate::legacy::HARDWARE_PARTS;

use super::{
    dtos::{
        BuyPartRequest,
        MachineResponse,
        PartBoughtResponse,
        ShopPartResponse,
    },
    repo,
};

//
// ==============================
// MACHINES
// ==============================
//

pub async fn machines(
    pool: &PgPool,
    user_id: i64,
) -> Result<Vec<MachineResponse>, ApiError> {
    let machines =
        repo::find_machines(pool, user_id)
            .await?
            .into_iter()
            .map(MachineResponse::from)
            .collect();

    Ok(machines)
}

//
// ==============================
// SHOP
// ==============================
//

/// Port de hardwareItens.php, que guardaba el catalogo en arrays estaticos de PHP.
pub fn shop() -> Vec<ShopPartResponse> {
    HARDWARE_PARTS
        .iter()
        .map(|part| ShopPartResponse {
            kind: part.kind,
            id: part.id,
            name: part.name,
            power: part.power,
            price: part.price,
        })
        .collect()
}

/// Comprar sustituye lo que hubiera en la maquina en vez de sumarse, asi que una pieza
/// mas barata es una bajada. El legado lo permitia y aqui tambien, pero no hasta el punto
/// de dejar fuera ficheros ya guardados o software ya cargado.
pub async fn buy(
    pool: &PgPool,
    user_id: i64,
    input: BuyPartRequest,
) -> Result<PartBoughtResponse, ApiError> {
    let part = HARDWARE_PARTS
        .iter()
        .find(|part| {
            part.kind == input.kind
                && part.id == input.part
        })
        .ok_or_else(|| {
            ApiError::BadRequest(
                "unknown part".into(),
            )
        })?;

    let mut tx = pool.begin().await?;

    let cash = repo::lock_account_cash(
        &mut tx,
        input.account,
        user_id,
    )
    .await?
    .unwrap_or(0);

    if cash < part.price {
        return Err(ApiError::BadRequest(
            "insufficient funds".into(),
        ));
    }

    let floor = match part.kind {
        "hdd" => {
            repo::disk_in_use(&mut tx, user_id)
                .await?
        }
        "ram" => {
            repo::memory_in_use(&mut tx, user_id)
                .await?
        }
        _ => 0,
    };

    if (part.power as i64) < floor {
        return Err(ApiError::BadRequest(
            "PART_TOO_SMALL".into(),
        ));
    }

    repo::charge_account(
        &mut tx,
        input.account,
        user_id,
        part.price,
    )
    .await?;

    repo::install_part(
        &mut tx,
        part.kind,
        part.power as f32,
        user_id,
    )
    .await?;

    repo::record_spend(
        &mut tx,
        user_id,
        part.price,
    )
    .await?;

    tx.commit().await?;

    Ok(PartBoughtResponse {
        kind: part.kind,
        name: part.name,
        power: part.power,
        paid: part.price,
    })
}

//
// ==============================
// SERVER STATS
// ==============================
//

/// Sin worker todavia no hay foto, y la pantalla lo dice en vez de romperse.
pub async fn server_stats(
    pool: &PgPool,
) -> Result<serde_json::Value, ApiError> {
    let payload =
        repo::find_server_stats(pool).await?;

    Ok(payload.unwrap_or_else(
        || serde_json::json!({}),
    ))
}

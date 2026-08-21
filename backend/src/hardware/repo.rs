use sqlx::{PgPool, Postgres, Transaction};

use super::models::MachineRow;

//
// ==============================
// MACHINES
// ==============================
//

pub async fn find_machines(
    pool: &PgPool,
    user_id: i64,
) -> Result<Vec<MachineRow>, sqlx::Error> {
    sqlx::query_as::<_, MachineRow>(
        r#"
        SELECT
            server_id,
            name,
            cpu,
            hdd,
            ram,
            net
        FROM hardware
        WHERE user_id = $1
          AND NOT is_npc
        ORDER BY server_id
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

//
// ==============================
// PAYMENT
// ==============================
//

pub async fn lock_account_cash(
    tx: &mut Transaction<'_, Postgres>,
    account: i64,
    user_id: i64,
) -> Result<Option<i64>, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        SELECT cash
        FROM bank_accounts
        WHERE bank_acc = $1
          AND bank_user = $2
        FOR UPDATE
        "#,
    )
    .bind(account)
    .bind(user_id)
    .fetch_optional(&mut **tx)
    .await
}

pub async fn charge_account(
    tx: &mut Transaction<'_, Postgres>,
    account: i64,
    user_id: i64,
    amount: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE bank_accounts
        SET cash = cash - $1
        WHERE bank_acc = $2
          AND bank_user = $3
        "#,
    )
    .bind(amount)
    .bind(account)
    .bind(user_id)
    .execute(&mut **tx)
    .await?;

    Ok(())
}

pub async fn record_spend(
    tx: &mut Transaction<'_, Postgres>,
    user_id: i64,
    amount: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE users_stats
        SET money_hardware = money_hardware + $1
        WHERE uid = $2
        "#,
    )
    .bind(amount)
    .bind(user_id)
    .execute(&mut **tx)
    .await?;

    Ok(())
}

//
// ==============================
// LIMITS
// ==============================
//

/// Lo que ya ocupa el disco: una pieza mas pequena que esto dejaria ficheros fuera.
pub async fn disk_in_use(
    tx: &mut Transaction<'_, Postgres>,
    user_id: i64,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        SELECT COALESCE(sum(soft_size), 0)::bigint
        FROM software
        WHERE user_id = $1
          AND NOT is_npc
        "#,
    )
    .bind(user_id)
    .fetch_one(&mut **tx)
    .await
}

/// Lo que ya consume la memoria por el software cargado.
pub async fn memory_in_use(
    tx: &mut Transaction<'_, Postgres>,
    user_id: i64,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        SELECT COALESCE(sum(s.soft_ram), 0)::bigint
        FROM software s

        JOIN software_running r
            ON r.soft_id = s.id

        WHERE r.user_id = $1
          AND NOT r.is_npc
        "#,
    )
    .bind(user_id)
    .fetch_one(&mut **tx)
    .await
}

//
// ==============================
// UPGRADE
// ==============================
//

/// Una pieza sustituye lo que hubiera, no se suma. El nombre de la columna sale del tipo
/// de pieza, que ya vino validado contra el catalogo.
pub async fn install_part(
    tx: &mut Transaction<'_, Postgres>,
    kind: &str,
    power: f32,
    user_id: i64,
) -> Result<(), sqlx::Error> {
    let statement = match kind {
        "cpu" => {
            r#"
            UPDATE hardware
            SET cpu = $1
            WHERE user_id = $2
              AND NOT is_npc
            "#
        }
        "ram" => {
            r#"
            UPDATE hardware
            SET ram = $1
            WHERE user_id = $2
              AND NOT is_npc
            "#
        }
        "hdd" => {
            r#"
            UPDATE hardware
            SET hdd = $1
            WHERE user_id = $2
              AND NOT is_npc
            "#
        }
        _ => {
            r#"
            UPDATE hardware
            SET net = $1
            WHERE user_id = $2
              AND NOT is_npc
            "#
        }
    };

    sqlx::query(statement)
        .bind(power)
        .bind(user_id)
        .execute(&mut **tx)
        .await?;

    Ok(())
}

//
// ==============================
// SERVER STATS
// ==============================
//

/// El worker deja aqui la foto del servidor. Es jsonb a proposito: el legado la calculaba
/// en cada visita y esto la deja crecer sin tocar el esquema.
pub async fn find_server_stats(
    pool: &PgPool,
) -> Result<Option<serde_json::Value>, sqlx::Error>
{
    sqlx::query_scalar(
        r#"
        SELECT payload
        FROM system_stats
        WHERE key = 'server'
        "#,
    )
    .fetch_optional(pool)
    .await
}

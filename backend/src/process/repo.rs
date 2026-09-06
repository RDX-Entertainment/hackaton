use chrono::{DateTime, Utc};
use sqlx::{PgPool, Postgres, Transaction};

use super::models::ProcessRow;

//
// ==============================
// QUEUE
// ==============================
//

pub async fn find_for_user(
    pool: &PgPool,
    user_id: i64,
) -> Result<Vec<ProcessRow>, sqlx::Error> {
    sqlx::query_as::<_, ProcessRow>(
        r#"
        SELECT
            pid,
            victim_id,
            action,
            info,
            info_str,
            time_ideal,
            time_worked,
            GREATEST(
                0,
                EXTRACT(EPOCH FROM (time_end - now()))::integer
            ) AS seconds_left,
            is_paused,
            completed_at,
            status,
            result_code,
            result_message
        FROM processes
        WHERE creator_id = $1
        ORDER BY time_start DESC
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

/// Leer una sola tarea por id conserva la comprobacion de propiedad de la cola. Es la
/// consulta ligera que usa la UI mientras una accion acaba de ser lanzada.
pub async fn find_one_for_user(
    pool: &PgPool,
    process_id: i64,
    user_id: i64,
) -> Result<Option<ProcessRow>, sqlx::Error> {
    sqlx::query_as::<_, ProcessRow>(
        r#"
        SELECT
            pid, victim_id, action, info, info_str, time_ideal, time_worked,
            GREATEST(0, EXTRACT(EPOCH FROM (time_end - now()))::integer) AS seconds_left,
            is_paused, completed_at, status, result_code, result_message
        FROM processes
        WHERE pid = $1 AND creator_id = $2
        "#,
    )
    .bind(process_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await
}

//
// ==============================
// START
// ==============================
//

pub async fn owns_software(
    pool: &PgPool,
    software_id: i64,
    user_id: i64,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        SELECT EXISTS(
            SELECT 1
            FROM software
            WHERE id = $1
              AND user_id = $2
              AND NOT is_npc
        )
        "#,
    )
    .bind(software_id)
    .bind(user_id)
    .fetch_one(pool)
    .await
}

/// El legado no dejaba encolar dos veces exactamente la misma tarea.
pub async fn duplicate_exists(
    pool: &PgPool,
    user_id: i64,
    action: i16,
    victim_id: i64,
    software_id: i64,
    info: &str,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        SELECT EXISTS(
            SELECT 1
            FROM processes
            WHERE creator_id = $1
              AND action = $2
              AND victim_id = $3
              AND soft_id = $4
              AND info = $5
              AND completed_at IS NULL
        )
        "#,
    )
    .bind(user_id)
    .bind(action)
    .bind(victim_id)
    .bind(software_id)
    .bind(info)
    .fetch_one(pool)
    .await
}

pub async fn insert(
    pool: &PgPool,
    user_id: i64,
    victim_id: i64,
    action: i16,
    software_id: i64,
    info: &str,
    seconds: i32,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        INSERT INTO processes(
            creator_id,
            victim_id,
            action,
            soft_id,
            info,
            info_str,
            time_end,
            time_ideal,
            -- Nace donde vive su programa, y se escribe ahora: mover el software despues no
            -- reescribe de donde salio esto.
            origin_server,
            -- Y a que maquina va. Esta ruta no nombra una: se pide contra un dueno, sin pasar
            -- por una direccion. Entonces va a su puerta principal, que es exactamente lo que
            -- pasaba antes sin escribirlo -- `victim_ip` resolvia la `game_ip`, que desde la
            -- fase 3 de direcciones es la de esa maquina. Se deja anotado en vez de deducirlo
            -- despues, porque para entonces el jugador puede haber cambiado cual manda.
            --
            -- El `info` es lo que distingue un NPC de un jugador en esta tabla desde el
            -- legado, y es la misma regla que aplica el worker al leerlo.
            victim_server
        )
        VALUES(
            $1,
            $2,
            $3,
            $4,
            $5,
            $5,
            now() + ($6::integer * interval '1 second'),
            $6,
            (SELECT server_id FROM software WHERE id = $4),
            (SELECT server_id FROM hardware
              WHERE user_id = $2
                AND is_npc = ($5 = 'npc')
                AND is_gateway)
        )
        RETURNING pid
        "#,
    )
    .bind(user_id)
    .bind(victim_id)
    .bind(action)
    .bind(software_id)
    .bind(info)
    .bind(seconds)
    .fetch_one(pool)
    .await
}

//
// ==============================
// DELETE
// ==============================
//

pub async fn delete(
    pool: &PgPool,
    process_id: i64,
    user_id: i64,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        r#"
        DELETE FROM processes
        WHERE pid = $1
          AND creator_id = $2
        "#,
    )
    .bind(process_id)
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}

//
// ==============================
// PAUSE
// ==============================
//

pub async fn lock_running_end(
    tx: &mut Transaction<'_, Postgres>,
    process_id: i64,
    user_id: i64,
) -> Result<Option<DateTime<Utc>>, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        SELECT time_end
        FROM processes
        WHERE pid = $1
          AND creator_id = $2
          AND NOT is_paused
        FOR UPDATE
        "#,
    )
    .bind(process_id)
    .bind(user_id)
    .fetch_optional(&mut **tx)
    .await
}

/// Lo que le quedaba por delante al pausar, que es lo que se guarda para devolverselo.
pub async fn seconds_until(
    tx: &mut Transaction<'_, Postgres>,
    end: DateTime<Utc>,
) -> Result<i32, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        SELECT GREATEST(
            0,
            EXTRACT(EPOCH FROM ($1::timestamptz - now()))::integer
        )
        "#,
    )
    .bind(end)
    .fetch_one(&mut **tx)
    .await
}

pub async fn pause(
    tx: &mut Transaction<'_, Postgres>,
    process_id: i64,
    user_id: i64,
    seconds_left: i32,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE processes
        SET is_paused = true
        WHERE pid = $1
        "#,
    )
    .bind(process_id)
    .execute(&mut **tx)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO processes_paused(
            pid,
            time_left,
            user_id
        )
        VALUES($1, $2, $3)

        ON CONFLICT(pid)
        DO UPDATE SET
            time_left = EXCLUDED.time_left,
            user_id = EXCLUDED.user_id,
            time_paused = now()
        "#,
    )
    .bind(process_id)
    .bind(seconds_left)
    .bind(user_id)
    .execute(&mut **tx)
    .await?;

    Ok(())
}

//
// ==============================
// RESUME
// ==============================
//

pub async fn lock_paused_seconds(
    tx: &mut Transaction<'_, Postgres>,
    process_id: i64,
    user_id: i64,
) -> Result<Option<i32>, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        SELECT pp.time_left
        FROM processes_paused pp

        JOIN processes p
            ON p.pid = pp.pid

        WHERE pp.pid = $1
          AND pp.user_id = $2
          AND p.creator_id = $2
          AND p.is_paused

        FOR UPDATE
        "#,
    )
    .bind(process_id)
    .bind(user_id)
    .fetch_optional(&mut **tx)
    .await
}

pub async fn resume(
    tx: &mut Transaction<'_, Postgres>,
    process_id: i64,
    seconds_left: i32,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE processes
        SET is_paused = false,
            time_start = now(),
            time_end = now() + ($1::integer * interval '1 second')
        WHERE pid = $2
        "#,
    )
    .bind(seconds_left)
    .bind(process_id)
    .execute(&mut **tx)
    .await?;

    sqlx::query(
        r#"
        DELETE FROM processes_paused
        WHERE pid = $1
        "#,
    )
    .bind(process_id)
    .execute(&mut **tx)
    .await?;

    Ok(())
}

use sqlx::PgPool;

use super::models::MissionRow;

//
// ==============================
// BOARD
// ==============================
//

/// El tablero de un jugador son sus propias misiones mas las que siguen abiertas.
pub async fn find_visible(
    pool: &PgPool,
    user_id: i64,
) -> Result<Vec<MissionRow>, sqlx::Error> {
    sqlx::query_as::<_, MissionRow>(
        r#"
        SELECT
            id,
            type AS mission_type,
            status,
            hirer,
            victim,
            info,
            info2,
            prize,
            level
        FROM missions
        WHERE user_id = $1
           OR status = 1
        ORDER BY date_generated DESC
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

//
// ==============================
// ACCEPT
// ==============================
//

pub async fn count_active(
    pool: &PgPool,
    user_id: i64,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM missions
        WHERE user_id = $1
          AND status = 2
        "#,
    )
    .bind(user_id)
    .fetch_one(pool)
    .await
}

pub async fn find_open_level(
    pool: &PgPool,
    mission_id: i64,
) -> Result<Option<i16>, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        SELECT level
        FROM missions
        WHERE id = $1
          AND status = 1
        "#,
    )
    .bind(mission_id)
    .fetch_optional(pool)
    .await
}

pub async fn best_cracker_version(
    pool: &PgPool,
    user_id: i64,
) -> Result<Option<i32>, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        SELECT MAX(soft_version)
        FROM software
        WHERE user_id = $1
          AND NOT is_npc
          AND soft_type = 1
        "#,
    )
    .bind(user_id)
    .fetch_one(pool)
    .await
}

pub async fn accept(
    pool: &PgPool,
    mission_id: i64,
    user_id: i64,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        r#"
        UPDATE missions
        SET status = 2,
            user_id = $1
        WHERE id = $2
          AND status = 1
        "#,
    )
    .bind(user_id)
    .bind(mission_id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}

//
// ==============================
// ABORT
// ==============================
//

/// El legado solo dejaba abandonar las misiones normales: de 50 en adelante son de la
/// linea argumental y no se sueltan.
pub async fn abort(
    pool: &PgPool,
    mission_id: i64,
    user_id: i64,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        r#"
        UPDATE missions
        SET status = 4
        WHERE id = $1
          AND user_id = $2
          AND status = 2
          AND type <= 49
        "#,
    )
    .bind(mission_id)
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}

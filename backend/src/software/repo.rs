use sqlx::PgPool;

use super::models::{
    RemoteSoftwareRow,
    ResearchHistoryRow,
    SoftwareRow,
    TransferSoftwareRow,
};

//
// ==============================
// SOFTWARE LOCAL
// ==============================
//

pub async fn find_user_software(
    pool: &PgPool,
    user_id: i64,
) -> Result<Vec<SoftwareRow>, sqlx::Error> {
    sqlx::query_as::<_, SoftwareRow>(
        r#"
        SELECT
            s.id,
            s.soft_name,
            s.soft_version,
            s.soft_size,
            s.soft_ram,
            s.soft_type,
            s.soft_hidden,
            s.soft_hidden_with,
            s.is_folder,
            (r.soft_id IS NOT NULL) AS running
        FROM software s

        LEFT JOIN software_running r
            ON r.soft_id = s.id

        WHERE s.user_id = $1
          AND NOT s.is_npc

        ORDER BY s.soft_name
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

pub async fn total_hdd(
    pool: &PgPool,
    user_id: i64,
) -> Result<Option<f32>, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        SELECT SUM(hdd)::real
        FROM hardware
        WHERE user_id = $1
          AND NOT is_npc
        "#,
    )
    .bind(user_id)
    .fetch_one(pool)
    .await
}

pub async fn used_hdd(
    pool: &PgPool,
    user_id: i64,
) -> Result<Option<i64>, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        SELECT SUM(soft_size)::bigint
        FROM software
        WHERE user_id = $1
          AND NOT is_npc
        "#,
    )
    .bind(user_id)
    .fetch_one(pool)
    .await
}

pub async fn create_software(
    pool: &PgPool,
    user_id: i64,
    name: &str,
    size: i32,
    ram: i32,
    soft_type: i16,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        INSERT INTO software(
            user_id,
            soft_name,
            soft_version,
            soft_size,
            soft_ram,
            soft_type
        )
        VALUES($1, $2, 1, $3, $4, $5)
        RETURNING id
        "#,
    )
    .bind(user_id)
    .bind(name)
    .bind(size)
    .bind(ram)
    .bind(soft_type)
    .fetch_one(pool)
    .await
}

//
// ==============================
// SOFTWARE REMOTO
// ==============================
//

pub async fn find_remote_software(
    pool: &PgPool,
    owner_id: i64,
    is_npc: bool,
) -> Result<Vec<RemoteSoftwareRow>, sqlx::Error> {
    sqlx::query_as::<_, RemoteSoftwareRow>(
        r#"
        SELECT
            id,
            soft_name,
            soft_version,
            soft_size,
            soft_ram,
            soft_type,
            is_folder
        FROM software
        WHERE user_id = $1
          AND is_npc = $2
          AND NOT soft_hidden
        ORDER BY soft_name
        "#,
    )
    .bind(owner_id)
    .bind(is_npc)
    .fetch_all(pool)
    .await
}

pub async fn find_transfer_software(
    pool: &PgPool,
    software_id: i64,
    owner_id: i64,
    owner_is_npc: bool,
) -> Result<Option<TransferSoftwareRow>, sqlx::Error> {
    sqlx::query_as::<_, TransferSoftwareRow>(
        r#"
        SELECT
            soft_type,
            soft_name,
            soft_size
        FROM software
        WHERE id = $1
          AND user_id = $2
          AND is_npc = $3
          AND NOT soft_hidden
        "#,
    )
    .bind(software_id)
    .bind(owner_id)
    .bind(owner_is_npc)
    .fetch_optional(pool)
    .await
}

pub async fn remote_software_exists(
    pool: &PgPool,
    software_id: i64,
    owner_id: i64,
    is_npc: bool,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        SELECT EXISTS(
            SELECT 1
            FROM software
            WHERE id = $1
              AND user_id = $2
              AND is_npc = $3
        )
        "#,
    )
    .bind(software_id)
    .bind(owner_id)
    .bind(is_npc)
    .fetch_one(pool)
    .await
}

//
// ==============================
// TRANSFER PROCESSES
// ==============================
//

pub async fn active_transfer_exists(
    pool: &PgPool,
    user_id: i64,
    action: i16,
    victim_id: i64,
    software_id: i64,
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
              AND completed_at IS NULL
        )
        "#,
    )
    .bind(user_id)
    .bind(action)
    .bind(victim_id)
    .bind(software_id)
    .fetch_one(pool)
    .await
}

pub async fn create_transfer_process(
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
            time_ideal
        )
        VALUES(
            $1,
            $2,
            $3,
            $4,
            $5,
            $5,
            now() + ($6::integer * interval '1 second'),
            $6
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

pub async fn create_delete_process(
    pool: &PgPool,
    user_id: i64,
    victim_id: i64,
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
            time_ideal
        )
        VALUES(
            $1,
            $2,
            3,
            $3,
            $4,
            $4,
            now() + ($5::integer * interval '1 second'),
            $5
        )
        RETURNING pid
        "#,
    )
    .bind(user_id)
    .bind(victim_id)
    .bind(software_id)
    .bind(info)
    .bind(seconds)
    .fetch_one(pool)
    .await
}

//
// ==============================
// RESEARCH
// ==============================
//

pub async fn research_history(
    pool: &PgPool,
    user_id: i64,
) -> Result<Vec<ResearchHistoryRow>, sqlx::Error> {
    sqlx::query_as::<_, ResearchHistoryRow>(
        r#"
        SELECT
            id,
            software_id,
            software_type,
            new_version,
            software_name,
            researched_at
        FROM research_history
        WHERE user_id = $1
        ORDER BY researched_at DESC
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

pub async fn local_software_exists(
    pool: &PgPool,
    user_id: i64,
    software_id: i64,
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

pub async fn active_research_exists(
    pool: &PgPool,
    user_id: i64,
    software_id: i64,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        SELECT EXISTS(
            SELECT 1
            FROM processes
            WHERE creator_id = $1
              AND action = 17
              AND soft_id = $2
              AND completed_at IS NULL
        )
        "#,
    )
    .bind(user_id)
    .bind(software_id)
    .fetch_one(pool)
    .await
}

pub async fn create_research_process(
    pool: &PgPool,
    user_id: i64,
    software_id: i64,
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
            time_ideal
        )
        VALUES(
            $1,
            0,
            17,
            $2,
            'local',
            'local',
            now() + ($3::integer * interval '1 second'),
            $3
        )
        RETURNING pid
        "#,
    )
    .bind(user_id)
    .bind(software_id)
    .bind(seconds)
    .fetch_one(pool)
    .await
}
use sqlx::{
    PgPool,
    Postgres,
    Transaction,
};

use super::models::{
    AnalyzeResultRow,
    BestAnalyzerRow,
    BookmarkRow,
    HistoryRow,
    InternetTargetRow,
    NpcRemoteRow,
    PlayerRemoteRow,
};

//
// ==============================
// TARGETS
// ==============================
//

pub async fn targets(
    pool: &PgPool,
) -> Result<Vec<InternetTargetRow>, sqlx::Error> {
    sqlx::query_as::<_, InternetTargetRow>(
        r#"
        SELECT
            n.id,
            n.npc_ip,
            n.npc_type,
            i.name,
            i.web
        FROM npc n
        LEFT JOIN npc_info i
            ON i.npc_id = n.id
        WHERE n.down_until <= now()
        ORDER BY
            n.npc_type,
            n.npc_ip
        LIMIT 200
        "#,
    )
    .fetch_all(pool)
    .await
}

//
// ==============================
// IP RESOLUTION
// ==============================
//

pub async fn find_npc_by_ip(
    pool: &PgPool,
    ip: i64,
) -> Result<Option<NpcRemoteRow>, sqlx::Error> {
    sqlx::query_as::<_, NpcRemoteRow>(
        r#"
        SELECT
            n.id,
            n.npc_type,
            n.down_until > now() AS down,
            COALESCE(i.name, '') AS name
        FROM npc n
        LEFT JOIN npc_info i
            ON i.npc_id = n.id
        WHERE n.npc_ip = $1
        "#,
    )
    .bind(ip)
    .fetch_optional(pool)
    .await
}

pub async fn find_player_by_ip(
    pool: &PgPool,
    ip: i64,
) -> Result<Option<PlayerRemoteRow>, sqlx::Error> {
    sqlx::query_as::<_, PlayerRemoteRow>(
        r#"
        SELECT
            id,
            login
        FROM users
        WHERE game_ip = $1
        "#,
    )
    .bind(ip)
    .fetch_optional(pool)
    .await
}

//
// ==============================
// NAVIGATION
// ==============================
//

pub async fn save_history(
    pool: &PgPool,
    user_id: i64,
    ip: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO internet_history(
            user_id,
            ip
        )
        VALUES($1, $2)

        ON CONFLICT(user_id, ip)
        DO UPDATE SET
            visited_at = now()
        "#,
    )
    .bind(user_id)
    .bind(ip)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn has_access(
    pool: &PgPool,
    user_id: i64,
    ip: i64,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        SELECT EXISTS(
            SELECT 1
            FROM internet_access
            WHERE user_id = $1
              AND ip = $2
        )
        "#,
    )
    .bind(user_id)
    .bind(ip)
    .fetch_one(pool)
    .await
}

pub async fn is_bookmarked(
    pool: &PgPool,
    user_id: i64,
    ip: i64,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        SELECT EXISTS(
            SELECT 1
            FROM internet_important
            WHERE user_id = $1
              AND ip = $2
        )
        "#,
    )
    .bind(user_id)
    .bind(ip)
    .fetch_one(pool)
    .await
}

pub async fn was_scanned(
    pool: &PgPool,
    user_id: i64,
    ip: i64,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        SELECT EXISTS(
            SELECT 1
            FROM internet_scan
            WHERE user_id = $1
              AND ip = $2
        )
        "#,
    )
    .bind(user_id)
    .bind(ip)
    .fetch_one(pool)
    .await
}

pub async fn analysis(
    pool: &PgPool,
    user_id: i64,
    ip: i64,
) -> Result<Option<AnalyzeResultRow>, sqlx::Error> {
    sqlx::query_as::<_, AnalyzeResultRow>(
        r#"
        SELECT
            cpu_min,
            cpu_max,
            ram_min,
            ram_max
        FROM analyze_results
        WHERE user_id = $1
          AND ip = $2
        "#,
    )
    .bind(user_id)
    .bind(ip)
    .fetch_optional(pool)
    .await
}

//
// ==============================
// LOGIN
// ==============================
//

pub async fn npc_password(
    pool: &PgPool,
    npc_id: i64,
) -> Result<String, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT npc_pass FROM npc WHERE id = $1",
    )
    .bind(npc_id)
    .fetch_one(pool)
    .await
}

pub async fn player_password(
    pool: &PgPool,
    user_id: i64,
) -> Result<String, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT game_pass FROM users WHERE id = $1",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await
}

pub async fn grant_access(
    pool: &PgPool,
    user_id: i64,
    ip: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO internet_access(
            user_id,
            ip,
            via_exploit
        )
        VALUES($1, $2, false)

        ON CONFLICT(user_id, ip)
        DO UPDATE SET
            granted_at = now()
        "#,
    )
    .bind(user_id)
    .bind(ip)
    .execute(pool)
    .await?;

    Ok(())
}

//
// ==============================
// HACK
// ==============================
//

pub async fn certification_level(
    pool: &PgPool,
    user_id: i64,
) -> Result<Option<i16>, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        SELECT cert_level
        FROM certifications
        WHERE user_id = $1
        "#,
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
}

pub async fn best_local_software_version(
    pool: &PgPool,
    user_id: i64,
    soft_type: i16,
) -> Result<Option<i32>, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        SELECT MAX(soft_version)
        FROM software
        WHERE user_id = $1
          AND soft_type = $2
          AND NOT soft_hidden
          AND NOT is_npc
        "#,
    )
    .bind(user_id)
    .bind(soft_type)
    .fetch_one(pool)
    .await
}

pub async fn best_remote_software_version(
    pool: &PgPool,
    owner_id: i64,
    soft_type: i16,
    is_npc: bool,
) -> Result<i32, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        SELECT COALESCE(MAX(soft_version), 0)
        FROM software
        WHERE user_id = $1
          AND soft_type = $2
          AND NOT soft_hidden
          AND is_npc = $3
        "#,
    )
    .bind(owner_id)
    .bind(soft_type)
    .bind(is_npc)
    .fetch_one(pool)
    .await
}

pub async fn active_hack_exists(
    pool: &PgPool,
    user_id: i64,
    action: i16,
    victim_id: i64,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        SELECT EXISTS(
            SELECT 1
            FROM processes
            WHERE creator_id = $1
              AND action = $2
              AND victim_id = $3
              AND completed_at IS NULL
        )
        "#,
    )
    .bind(user_id)
    .bind(action)
    .bind(victim_id)
    .fetch_one(pool)
    .await
}

pub async fn create_hack_process(
    pool: &PgPool,
    user_id: i64,
    victim_id: i64,
    action: i16,
    info: &str,
    seconds: i32,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        INSERT INTO processes(
            creator_id,
            victim_id,
            action,
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
            $4,
            now() + ($5::integer * interval '1 second'),
            $5
        )
        RETURNING pid
        "#,
    )
    .bind(user_id)
    .bind(victim_id)
    .bind(action)
    .bind(info)
    .bind(seconds)
    .fetch_one(pool)
    .await
}

//
// ==============================
// REMOTE LOG
// ==============================
//

pub async fn remote_log(
    pool: &PgPool,
    owner_id: i64,
    is_npc: bool,
) -> Result<Option<String>, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        SELECT text
        FROM log
        WHERE user_id = $1
          AND is_npc = $2
        "#,
    )
    .bind(owner_id)
    .bind(is_npc)
    .fetch_optional(pool)
    .await
}

pub async fn create_log_edit_process(
    tx: &mut Transaction<'_, Postgres>,
    user_id: i64,
    victim_id: i64,
    info: &str,
    seconds: i32,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        INSERT INTO processes(
            creator_id,
            victim_id,
            action,
            info,
            info_str,
            time_end,
            time_ideal
        )
        VALUES(
            $1,
            $2,
            8,
            $3,
            $3,
            now() + ($4::integer * interval '1 second'),
            $4
        )
        RETURNING pid
        "#,
    )
    .bind(user_id)
    .bind(victim_id)
    .bind(info)
    .bind(seconds)
    .fetch_one(&mut **tx)
    .await
}

pub async fn create_log_edit(
    tx: &mut Transaction<'_, Postgres>,
    victim_id: i64,
    is_npc: bool,
    editor_id: i64,
    text: &str,
    process_id: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO log_edit(
            vic_id,
            is_npc,
            editor_id,
            log_text,
            process_id
        )
        VALUES($1, $2, $3, $4, $5)
        "#,
    )
    .bind(victim_id)
    .bind(is_npc)
    .bind(editor_id)
    .bind(text)
    .bind(process_id)
    .execute(&mut **tx)
    .await?;

    Ok(())
}

//
// ==============================
// ANALYZER
// ==============================
//

pub async fn best_running_analyzer(
    pool: &PgPool,
    user_id: i64,
) -> Result<Option<BestAnalyzerRow>, sqlx::Error> {
    sqlx::query_as::<_, BestAnalyzerRow>(
        r#"
        SELECT
            s.id,
            s.soft_version
        FROM software s
        JOIN software_running r
            ON r.soft_id = s.id
        WHERE s.user_id = $1
          AND NOT s.is_npc
          AND s.soft_type = 16
        ORDER BY s.soft_version DESC
        LIMIT 1
        "#,
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
}

pub async fn active_analyze_exists(
    pool: &PgPool,
    user_id: i64,
    victim_id: i64,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        SELECT EXISTS(
            SELECT 1
            FROM processes
            WHERE creator_id = $1
              AND action = 23
              AND victim_id = $2
              AND completed_at IS NULL
        )
        "#,
    )
    .bind(user_id)
    .bind(victim_id)
    .fetch_one(pool)
    .await
}

pub async fn create_analyze_process(
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
            23,
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
// BOOKMARKS
// ==============================
//

pub async fn bookmarks(
    pool: &PgPool,
    user_id: i64,
) -> Result<Vec<BookmarkRow>, sqlx::Error> {
    sqlx::query_as::<_, BookmarkRow>(
        r#"
        SELECT
            b.ip,
            COALESCE(i.name, us.login, '') AS name
        FROM internet_important b

        LEFT JOIN npc n
            ON n.npc_ip = b.ip

        LEFT JOIN npc_info i
            ON i.npc_id = n.id

        LEFT JOIN users us
            ON us.game_ip = b.ip

        WHERE b.user_id = $1
        ORDER BY b.id
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

pub async fn add_bookmark(
    pool: &PgPool,
    user_id: i64,
    ip: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO internet_important(
            user_id,
            ip
        )
        VALUES($1, $2)
        ON CONFLICT DO NOTHING
        "#,
    )
    .bind(user_id)
    .bind(ip)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn remove_bookmark(
    pool: &PgPool,
    user_id: i64,
    ip: i64,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        r#"
        DELETE FROM internet_important
        WHERE user_id = $1
          AND ip = $2
        "#,
    )
    .bind(user_id)
    .bind(ip)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

//
// ==============================
// HISTORY
// ==============================
//

pub async fn history(
    pool: &PgPool,
    user_id: i64,
) -> Result<Vec<HistoryRow>, sqlx::Error> {
    sqlx::query_as::<_, HistoryRow>(
        r#"
        SELECT
            h.ip,
            h.visited_at,
            COALESCE(i.name, us.login, '') AS name
        FROM internet_history h

        LEFT JOIN npc n
            ON n.npc_ip = h.ip

        LEFT JOIN npc_info i
            ON i.npc_id = n.id

        LEFT JOIN users us
            ON us.game_ip = h.ip

        WHERE h.user_id = $1

        ORDER BY h.visited_at DESC

        LIMIT 5
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}
use sqlx::{
    PgPool,
    Postgres,
    Transaction,
};

use super::models::{
    AdminUserRow,
    CompromisedListEntry,
    DashboardUserRow,
    PlayerLogEntry,
    ProfileRow,
    SettingsRow,
};

//
// ==============================
// DASHBOARD
// ==============================
//

pub async fn find_dashboard_user(
    pool: &PgPool,
    user_id: i64,
) -> Result<Option<DashboardUserRow>, sqlx::Error> {
    sqlx::query_as::<_, DashboardUserRow>(
        r#"
        SELECT
            id,
            login,
            email,
            game_ip,
            premium
        FROM users
        WHERE id = $1
        "#,
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
}

pub async fn total_money(
    pool: &PgPool,
    user_id: i64,
) -> Result<Option<i64>, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        -- SUM sobre bigint devuelve numeric en PostgreSQL, que no entra en un i64
        -- sin ayuda: el cast evita un ColumnDecode en cuanto el jugador abre su
        -- primera cuenta.
        SELECT SUM(cash)::bigint
        FROM bank_accounts
        WHERE bank_user = $1
        "#,
    )
    .bind(user_id)
    .fetch_one(pool)
    .await
}

pub async fn process_count(
    pool: &PgPool,
    user_id: i64,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM processes
        WHERE creator_id = $1
        "#,
    )
    .bind(user_id)
    .fetch_one(pool)
    .await
}

pub async fn unread_mail_count(
    pool: &PgPool,
    user_id: i64,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM mails
        WHERE recipient_id = $1
          AND NOT is_read
          AND NOT is_deleted
        "#,
    )
    .bind(user_id)
    .fetch_one(pool)
    .await
}

//
// ==============================
// LOGS
// ==============================
//

pub async fn player_log(
    pool: &PgPool,
    user_id: i64,
) -> Result<Vec<PlayerLogEntry>, sqlx::Error> {
    sqlx::query_as::<_, PlayerLogEntry>(
        r#"
        SELECT
            id,
            text
        FROM log
        WHERE user_id = $1
        ORDER BY id DESC
        LIMIT 200
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

//
// ==============================
// SETTINGS
// ==============================
//

pub async fn find_settings(
    pool: &PgPool,
    user_id: i64,
) -> Result<SettingsRow, sqlx::Error> {
    sqlx::query_as::<_, SettingsRow>(
        r#"
        SELECT
            login,
            email,
            premium,
            last_login
        FROM users
        WHERE id = $1
        "#,
    )
    .bind(user_id)
    .fetch_one(pool)
    .await
}

pub async fn lock_password(
    tx: &mut Transaction<'_, Postgres>,
    user_id: i64,
) -> Result<String, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        SELECT password
        FROM users
        WHERE id = $1
        FOR UPDATE
        "#,
    )
    .bind(user_id)
    .fetch_one(&mut **tx)
    .await
}

pub async fn email_exists_for_other_user(
    tx: &mut Transaction<'_, Postgres>,
    email: &str,
    user_id: i64,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        SELECT EXISTS(
            SELECT 1
            FROM users
            WHERE email = $1
              AND id <> $2
        )
        "#,
    )
    .bind(email)
    .bind(user_id)
    .fetch_one(&mut **tx)
    .await
}

pub async fn update_email(
    tx: &mut Transaction<'_, Postgres>,
    user_id: i64,
    email: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE users
        SET email = $1
        WHERE id = $2
        "#,
    )
    .bind(email)
    .bind(user_id)
    .execute(&mut **tx)
    .await?;

    Ok(())
}

pub async fn update_password(
    tx: &mut Transaction<'_, Postgres>,
    user_id: i64,
    password_hash: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE users
        SET password = $1
        WHERE id = $2
        "#,
    )
    .bind(password_hash)
    .bind(user_id)
    .execute(&mut **tx)
    .await?;

    Ok(())
}

pub async fn insert_account_settings_log(
    tx: &mut Transaction<'_, Postgres>,
    user_id: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO log(user_id, text)
        VALUES($1, 'Account settings updated')
        "#,
    )
    .bind(user_id)
    .execute(&mut **tx)
    .await?;

    Ok(())
}

//
// ==============================
// CERTIFICATIONS
// ==============================
//

/// El nivel de certificacion **con lo que el jugador ya ha demostrado saber hacer**.
///
/// Los dos primeros niveles dejan de aprobarse con dos llamadas seguidas --empezar y
/// terminar, sin nada en medio-- y pasan a aprobarse haciendo: el 1 al tener un programa
/// compilado, el 2 al haber entrado en una maquina. Es lo mismo que hace A3 con el paso del
/// tutorial y por el mismo motivo: **un hecho no se desincroniza**, y quien llegue por otro
/// camino tambien lo ha demostrado.
///
/// Lo guardado sigue mandando hacia arriba --los niveles 3 y siguientes se compran, y un
/// administrador puede subirlo a mano-- porque `GREATEST` se queda con el mayor de los dos.
/// Y se queda con el que no es nulo, asi que una cuenta sin fila en `certifications` tampoco
/// se queda en cero por un `NULL`.
pub async fn effective_certification_level(
    pool: &PgPool,
    user_id: i64,
) -> Result<i16, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        SELECT GREATEST(
            (SELECT cert_level FROM certifications WHERE user_id = $1),
            (CASE
                WHEN EXISTS(SELECT 1 FROM machine_accesses WHERE attacker_id = $1)
                  OR EXISTS(SELECT 1 FROM internet_access WHERE user_id = $1) THEN 2
                WHEN EXISTS(SELECT 1 FROM software
                             WHERE user_id = $1 AND NOT is_npc AND NOT is_folder) THEN 1
                ELSE 0
             END)::smallint
        )
        "#,
    )
    .bind(user_id)
    .fetch_one(pool)
    .await
}

/// El nivel **guardado**, sin mirar lo que el jugador ha hecho.
///
/// Queda para quien tiene que escribir en esa columna. Para decidir si alguien puede hacer
/// algo se usa `effective_certification_level`.
pub async fn certification_level(
    pool: &PgPool,
    user_id: i64,
) -> Result<i16, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        SELECT cert_level
        FROM certifications
        WHERE user_id = $1
        "#,
    )
    .bind(user_id)
    .fetch_one(pool)
    .await
}

pub async fn user_learning(
    pool: &PgPool,
    user_id: i64,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        SELECT learning
        FROM users_learning
        WHERE user_id = $1
        "#,
    )
    .bind(user_id)
    .fetch_one(pool)
    .await
}

pub async fn active_puzzle(
    pool: &PgPool,
    user_id: i64,
) -> Result<Option<i64>, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        SELECT puzzle_id
        FROM users_puzzle
        WHERE user_id = $1
        "#,
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
}

pub async fn lock_certification_level(
    tx: &mut Transaction<'_, Postgres>,
    user_id: i64,
) -> Result<i16, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        SELECT cert_level
        FROM certifications
        WHERE user_id = $1
        FOR UPDATE
        "#,
    )
    .bind(user_id)
    .fetch_one(&mut **tx)
    .await
}

pub async fn lock_learning(
    tx: &mut Transaction<'_, Postgres>,
    user_id: i64,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        SELECT learning
        FROM users_learning
        WHERE user_id = $1
        FOR UPDATE
        "#,
    )
    .bind(user_id)
    .fetch_one(&mut **tx)
    .await
}

pub async fn lock_bank_cash(
    tx: &mut Transaction<'_, Postgres>,
    user_id: i64,
    account: i64,
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

pub async fn debit_bank_account(
    tx: &mut Transaction<'_, Postgres>,
    user_id: i64,
    account: i64,
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

pub async fn set_learning(
    tx: &mut Transaction<'_, Postgres>,
    user_id: i64,
    learning: bool,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE users_learning
        SET learning = $1
        WHERE user_id = $2
        "#,
    )
    .bind(learning)
    .bind(user_id)
    .execute(&mut **tx)
    .await?;

    Ok(())
}

pub async fn set_puzzle(
    tx: &mut Transaction<'_, Postgres>,
    user_id: i64,
    puzzle_id: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE users_puzzle
        SET puzzle_id = $1
        WHERE user_id = $2
        "#,
    )
    .bind(puzzle_id)
    .bind(user_id)
    .execute(&mut **tx)
    .await?;

    Ok(())
}

pub async fn lock_active_puzzle(
    tx: &mut Transaction<'_, Postgres>,
    user_id: i64,
) -> Result<Option<i64>, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        SELECT puzzle_id
        FROM users_puzzle
        WHERE user_id = $1
        FOR UPDATE
        "#,
    )
    .bind(user_id)
    .fetch_optional(&mut **tx)
    .await
}

pub async fn set_certification_level(
    tx: &mut Transaction<'_, Postgres>,
    user_id: i64,
    level: i16,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE certifications
        SET cert_level = $1
        WHERE user_id = $2
        "#,
    )
    .bind(level)
    .bind(user_id)
    .execute(&mut **tx)
    .await?;

    Ok(())
}

//
// ==============================
// PROFILE
// ==============================
//

pub async fn find_profile(
    pool: &PgPool,
    user_id: i64,
) -> Result<ProfileRow, sqlx::Error> {
    sqlx::query_as::<_, ProfileRow>(
        r#"
        SELECT
            u.id,
            u.login,
            u.game_ip,
            p.biography,
            p.web,
            COALESCE(rk.rank, -1)          AS rank,
            COALESCE(st.hack_count, 0)     AS hack_count,
            COALESCE(st.money_hardware, 0) AS money_hardware,
            COALESCE(st.exp, 0)            AS exp,
            st.date_joined
        FROM users u
        JOIN player_profiles p
            ON p.user_id = u.id
        LEFT JOIN ranking_user rk
            ON rk.user_id = u.id
        LEFT JOIN users_stats st
            ON st.uid = u.id
        WHERE u.id = $1
        "#,
    )
    .bind(user_id)
    .fetch_one(pool)
    .await
}

/// `web` en None deja la pagina como esta: guardar la biografia no la borra.
pub async fn upsert_profile(
    pool: &PgPool,
    user_id: i64,
    biography: &str,
    web: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO player_profiles(
            user_id,
            biography,
            web
        )
        VALUES($1, $2, COALESCE($3, ''))

        ON CONFLICT(user_id)
        DO UPDATE SET
            biography = EXCLUDED.biography,
            web = COALESCE($3, player_profiles.web),
            updated_at = now()
        "#,
    )
    .bind(user_id)
    .bind(biography)
    .bind(web)
    .execute(pool)
    .await?;

    Ok(())
}

//
// ==============================
// COMPROMISED LISTS
// ==============================
//

pub async fn compromised_lists(
    pool: &PgPool,
    user_id: i64,
) -> Result<Vec<CompromisedListEntry>, sqlx::Error> {
    sqlx::query_as::<_, CompromisedListEntry>(
        r#"
        SELECT
            id,
            ip,
            login,
            hacked_at,
            virus_id
        FROM compromised_lists
        WHERE user_id = $1
        ORDER BY hacked_at DESC
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

pub async fn insert_compromised_list(
    pool: &PgPool,
    user_id: i64,
    ip: i64,
    login: &str,
    password: &str,
    virus_id: i64,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        INSERT INTO compromised_lists(
            user_id,
            ip,
            login,
            password,
            virus_id
        )
        VALUES($1, $2, $3, $4, $5)
        RETURNING id
        "#,
    )
    .bind(user_id)
    .bind(ip)
    .bind(login)
    .bind(password)
    .bind(virus_id)
    .fetch_one(pool)
    .await
}

pub async fn delete_compromised_list(
    pool: &PgPool,
    user_id: i64,
    id: i64,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        r#"
        DELETE FROM compromised_lists
        WHERE id = $1
          AND user_id = $2
        "#,
    )
    .bind(id)
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}
//
// ==============================
// ADMIN
// ==============================
//

pub async fn find_all_users(
    pool: &PgPool,
) -> Result<Vec<AdminUserRow>, sqlx::Error> {
    sqlx::query_as::<_, AdminUserRow>(
        r#"
        SELECT
            u.id,
            u.login,
            u.email,
            u.game_ip,
            u.is_admin,
            u.premium,
            u.last_login,
            (
                SELECT count(*)
                FROM hardware_components c
                WHERE c.user_id = u.id
            ) AS components
        FROM users u
        ORDER BY u.id
        "#,
    )
    .fetch_all(pool)
    .await
}

pub async fn set_admin(
    pool: &PgPool,
    id: i64,
    admin: bool,
) -> Result<u64, sqlx::Error> {
    let done = sqlx::query("UPDATE users SET is_admin = $2 WHERE id = $1")
        .bind(id)
        .bind(admin)
        .execute(pool)
        .await?;

    Ok(done.rows_affected())
}

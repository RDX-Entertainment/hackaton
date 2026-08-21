use sqlx::{PgPool, Postgres, Transaction};

use super::models::{
    ClanMemberRow,
    ClanRequestRow,
    ClanRow,
    FriendRow,
    MyClanRow,
    PendingRequestRow,
    RankingRow,
};

//
// ==============================
// RANKING
// ==============================
//

pub async fn find_ranking(
    pool: &PgPool,
    limit: i64,
) -> Result<Vec<RankingRow>, sqlx::Error> {
    sqlx::query_as::<_, RankingRow>(
        r#"
        SELECT
            r.rank,
            u.id,
            u.login
        FROM ranking_user r

        JOIN users u
            ON u.id = r.user_id

        ORDER BY r.rank
        LIMIT $1
        "#,
    )
    .bind(limit)
    .fetch_all(pool)
    .await
}

//
// ==============================
// FRIENDS
// ==============================
//

/// La amistad se guarda en las dos direcciones, asi que el CASE devuelve siempre al otro.
pub async fn find_friends(
    pool: &PgPool,
    user_id: i64,
) -> Result<Vec<FriendRow>, sqlx::Error> {
    sqlx::query_as::<_, FriendRow>(
        r#"
        SELECT
            u.id,
            u.login
        FROM users_friends f

        JOIN users u
            ON u.id = CASE
                WHEN f.user_id = $1 THEN f.friend_id
                ELSE f.user_id
            END

        WHERE f.user_id = $1
           OR f.friend_id = $1
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

/// Devuelve None cuando el destinatario no existe o ya habia una solicitud.
pub async fn insert_friend_request(
    pool: &PgPool,
    target_id: i64,
    requester_id: i64,
) -> Result<Option<i64>, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        INSERT INTO friend_requests(
            user_id,
            requested_by
        )
        SELECT $1, $2
        WHERE EXISTS(
            SELECT 1
            FROM users
            WHERE id = $1
        )
        ON CONFLICT DO NOTHING
        RETURNING id
        "#,
    )
    .bind(target_id)
    .bind(requester_id)
    .fetch_optional(pool)
    .await
}

/// Aceptar consume la solicitud y devuelve quien la habia mandado.
pub async fn take_friend_request(
    tx: &mut Transaction<'_, Postgres>,
    request_id: i64,
    user_id: i64,
) -> Result<Option<i64>, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        DELETE FROM friend_requests
        WHERE id = $1
          AND user_id = $2
        RETURNING requested_by
        "#,
    )
    .bind(request_id)
    .bind(user_id)
    .fetch_optional(&mut **tx)
    .await
}

pub async fn insert_friendship(
    tx: &mut Transaction<'_, Postgres>,
    user_id: i64,
    friend_id: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO users_friends(
            user_id,
            friend_id
        )
        VALUES($1, $2), ($2, $1)
        ON CONFLICT DO NOTHING
        "#,
    )
    .bind(user_id)
    .bind(friend_id)
    .execute(&mut **tx)
    .await?;

    Ok(())
}

//
// ==============================
// CLAN LIST
// ==============================
//

pub async fn find_clans(
    pool: &PgPool,
) -> Result<Vec<ClanRow>, sqlx::Error> {
    sqlx::query_as::<_, ClanRow>(
        r#"
        SELECT
            c.id,
            c.clan_ip,
            c.name,
            c.nick,
            c.description,
            c.slots_total,
            c.money,
            c.power,
            COUNT(m.user_id) AS members
        FROM clans c

        LEFT JOIN clan_members m
            ON m.clan_id = c.id

        GROUP BY c.id
        ORDER BY c.power DESC, c.name
        "#,
    )
    .fetch_all(pool)
    .await
}

//
// ==============================
// OWN CLAN
// ==============================
//

pub async fn find_my_clan(
    pool: &PgPool,
    user_id: i64,
) -> Result<Option<MyClanRow>, sqlx::Error> {
    sqlx::query_as::<_, MyClanRow>(
        r#"
        SELECT
            c.id,
            c.name,
            c.nick,
            c.description,
            c.slots_total,
            c.money,
            c.power,
            m.auth_level
        FROM clan_members m

        JOIN clans c
            ON c.id = m.clan_id

        WHERE m.user_id = $1
        "#,
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
}

pub async fn find_members(
    pool: &PgPool,
    clan_id: i64,
) -> Result<Vec<ClanMemberRow>, sqlx::Error> {
    sqlx::query_as::<_, ClanMemberRow>(
        r#"
        SELECT
            u.id,
            u.login,
            m.auth_level
        FROM clan_members m

        JOIN users u
            ON u.id = m.user_id

        WHERE m.clan_id = $1

        ORDER BY m.auth_level DESC, m.member_since
        "#,
    )
    .bind(clan_id)
    .fetch_all(pool)
    .await
}

//
// ==============================
// MEMBERSHIP
// ==============================
//

pub async fn in_any_clan(
    pool: &PgPool,
    user_id: i64,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        SELECT EXISTS(
            SELECT 1
            FROM clan_members
            WHERE user_id = $1
        )
        "#,
    )
    .bind(user_id)
    .fetch_one(pool)
    .await
}

pub async fn in_any_clan_tx(
    tx: &mut Transaction<'_, Postgres>,
    user_id: i64,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        SELECT EXISTS(
            SELECT 1
            FROM clan_members
            WHERE user_id = $1
        )
        "#,
    )
    .bind(user_id)
    .fetch_one(&mut **tx)
    .await
}

//
// ==============================
// CLAN CREATION
// ==============================
//

pub async fn insert_clan(
    tx: &mut Transaction<'_, Postgres>,
    clan_ip: i64,
    name: &str,
    nick: &str,
    description: &str,
    created_by: i64,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        INSERT INTO clans(
            clan_ip,
            name,
            nick,
            description,
            created_by
        )
        VALUES($1, $2, $3, $4, $5)
        RETURNING id
        "#,
    )
    .bind(clan_ip)
    .bind(name)
    .bind(nick)
    .bind(description)
    .bind(created_by)
    .fetch_one(&mut **tx)
    .await
}

/// Quien funda entra como mando: auth_level 3 es el rango que da el legado al creador.
pub async fn insert_founder(
    tx: &mut Transaction<'_, Postgres>,
    clan_id: i64,
    user_id: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO clan_members(
            clan_id,
            user_id,
            auth_level,
            hierarchy
        )
        VALUES($1, $2, 3, 0)
        "#,
    )
    .bind(clan_id)
    .bind(user_id)
    .execute(&mut **tx)
    .await?;

    Ok(())
}

//
// ==============================
// JOIN REQUESTS
// ==============================
//

pub async fn insert_join_request(
    pool: &PgPool,
    clan_id: i64,
    user_id: i64,
    message: &str,
) -> Result<Option<i64>, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        INSERT INTO clan_requests(
            clan_id,
            user_id,
            message
        )
        SELECT $1, $2, $3
        WHERE EXISTS(
            SELECT 1
            FROM clans
            WHERE id = $1
        )
        ON CONFLICT DO NOTHING
        RETURNING id
        "#,
    )
    .bind(clan_id)
    .bind(user_id)
    .bind(message)
    .fetch_optional(pool)
    .await
}

/// Solo ve las solicitudes quien manda en el clan: auth_level 2 para arriba.
pub async fn find_requests_for_manager(
    pool: &PgPool,
    user_id: i64,
) -> Result<Vec<ClanRequestRow>, sqlx::Error> {
    sqlx::query_as::<_, ClanRequestRow>(
        r#"
        SELECT
            r.id,
            r.message,
            r.asked_at AS created_at,
            applicant.login
        FROM clan_requests r

        JOIN clan_members manager
            ON manager.clan_id = r.clan_id

        JOIN users applicant
            ON applicant.id = r.user_id

        WHERE manager.user_id = $1
          AND manager.auth_level >= 2

        ORDER BY r.asked_at
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

pub async fn lock_request_for_manager(
    tx: &mut Transaction<'_, Postgres>,
    request_id: i64,
    user_id: i64,
) -> Result<Option<PendingRequestRow>, sqlx::Error>
{
    sqlx::query_as::<_, PendingRequestRow>(
        r#"
        SELECT
            r.clan_id,
            r.user_id,
            c.slots_total
        FROM clan_requests r

        JOIN clans c
            ON c.id = r.clan_id

        JOIN clan_members a
            ON a.clan_id = r.clan_id

        WHERE r.id = $1
          AND a.user_id = $2
          AND a.auth_level >= 2

        FOR UPDATE
        "#,
    )
    .bind(request_id)
    .bind(user_id)
    .fetch_optional(&mut **tx)
    .await
}

pub async fn count_members(
    tx: &mut Transaction<'_, Postgres>,
    clan_id: i64,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM clan_members
        WHERE clan_id = $1
        "#,
    )
    .bind(clan_id)
    .fetch_one(&mut **tx)
    .await
}

pub async fn insert_member(
    tx: &mut Transaction<'_, Postgres>,
    clan_id: i64,
    user_id: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO clan_members(
            clan_id,
            user_id
        )
        VALUES($1, $2)
        "#,
    )
    .bind(clan_id)
    .bind(user_id)
    .execute(&mut **tx)
    .await?;

    Ok(())
}

pub async fn delete_request(
    tx: &mut Transaction<'_, Postgres>,
    request_id: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        DELETE FROM clan_requests
        WHERE id = $1
        "#,
    )
    .bind(request_id)
    .execute(&mut **tx)
    .await?;

    Ok(())
}

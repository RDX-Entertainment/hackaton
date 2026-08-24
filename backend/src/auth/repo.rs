use sqlx::{PgPool, Postgres, Transaction};

use super::models::{CredentialsRow, UserRow};

//
// ==============================
// LOOKUPS
// ==============================
//

// Acepta indistintamente el nombre de usuario o el correo: la pantalla de entrada
// pide el correo, y no hay motivo para que quien recuerde su login no pueda usarlo.
// Ambas columnas son unicas, asi que no hay ambiguedad posible.
pub async fn find_credentials_by_identifier(
    pool: &PgPool,
    identifier: &str,
) -> Result<Option<CredentialsRow>, sqlx::Error> {
    sqlx::query_as::<_, CredentialsRow>(
        r#"
        SELECT
            id,
            login,
            email,
            game_ip,
            premium,
            password
        FROM users
        WHERE login = $1
           OR email = $1
        "#,
    )
    .bind(identifier)
    .fetch_optional(pool)
    .await
}

pub async fn find_user_by_id(
    pool: &PgPool,
    id: i64,
) -> Result<Option<UserRow>, sqlx::Error> {
    sqlx::query_as::<_, UserRow>(
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
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn login_or_email_taken(
    pool: &PgPool,
    login: &str,
    email: &str,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        SELECT EXISTS(
            SELECT 1
            FROM users
            WHERE login = $1
               OR email = $2
        )
        "#,
    )
    .bind(login)
    .bind(email)
    .fetch_one(pool)
    .await
}

//
// ==============================
// SESSION
// ==============================
//

pub async fn touch_last_login(
    pool: &PgPool,
    id: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE users
        SET last_login = now()
        WHERE id = $1
        "#,
    )
    .bind(id)
    .execute(pool)
    .await?;

    Ok(())
}

//
// ==============================
// REGISTRATION
// ==============================
//

pub async fn insert_user(
    tx: &mut Transaction<'_, Postgres>,
    login: &str,
    password_hash: &str,
    email: &str,
    game_pass: &str,
    game_ip: i64,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        INSERT INTO users(
            login,
            password,
            email,
            game_pass,
            game_ip,
            real_ip,
            home_ip
        )
        VALUES($1, $2, $3, $4, $5, $5, $5)
        RETURNING id
        "#,
    )
    .bind(login)
    .bind(password_hash)
    .bind(email)
    .bind(game_pass)
    .bind(game_ip)
    .fetch_one(&mut **tx)
    .await
}

/// Las filas que todo jugador nuevo necesita. El legado las creaba desperdigadas por
/// register.php; aqui van juntas y dentro de la misma transaccion que el alta.
const BOOTSTRAP: &[&str] = &[
    "INSERT INTO users_stats(uid) VALUES($1)",
    "INSERT INTO hardware(user_id,name) VALUES($1,'Server #1')",
    // El equipo de arranque sale del kit marcado por defecto, que un administrador puede
    // cambiar desde el panel. Sin componentes la maquina nace a cero y no podria ni cargar
    // software: la capacidad es la suma de lo instalado. Si no hay kit por defecto esto no
    // inserta nada y el alta sigue adelante con la maquina vacia.
    "INSERT INTO hardware_components(part_id,user_id,installed_in)
     SELECT ki.part_id,$1,(SELECT server_id FROM hardware WHERE user_id=$1 AND NOT is_npc ORDER BY server_id LIMIT 1)
     FROM hardware_kits k
     JOIN hardware_kit_items ki ON ki.kit_id=k.id
     CROSS JOIN generate_series(1,ki.amount)
     WHERE k.is_default",
    "UPDATE hardware h SET cpu=COALESCE(s.cpu,0),ram=COALESCE(s.ram,0),hdd=COALESCE(s.hdd,0),net=COALESCE(s.net,0)
     FROM (SELECT c.installed_in AS server_id,
                  sum(k.power) FILTER (WHERE k.kind='cpu') AS cpu,
                  sum(k.power) FILTER (WHERE k.kind='ram') AS ram,
                  sum(k.power) FILTER (WHERE k.kind='hdd') AS hdd,
                  sum(k.power) FILTER (WHERE k.kind='net') AS net
           FROM hardware_components c JOIN hardware_parts k ON k.id=c.part_id
           WHERE c.user_id=$1 AND c.installed_in IS NOT NULL
           GROUP BY c.installed_in) s
     WHERE h.server_id=s.server_id",
    "INSERT INTO log(user_id,text) VALUES($1,'localhost installed current operating system')",
    "INSERT INTO cache(user_id) VALUES($1)",
    "INSERT INTO cache_profile(user_id) VALUES($1)",
    "INSERT INTO ranking_user(user_id,rank) VALUES($1,-1)",
    "INSERT INTO certifications(user_id) VALUES($1)",
    "INSERT INTO users_puzzle(user_id) VALUES($1)",
    "INSERT INTO users_learning(user_id) VALUES($1)",
    "INSERT INTO users_language(user_id) VALUES($1)",
    "INSERT INTO player_profiles(user_id) VALUES($1)",
];

pub async fn bootstrap_player(
    tx: &mut Transaction<'_, Postgres>,
    id: i64,
) -> Result<(), sqlx::Error> {
    for statement in BOOTSTRAP {
        sqlx::query(statement)
            .bind(id)
            .execute(&mut **tx)
            .await?;
    }

    Ok(())
}

pub async fn find_user_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    id: i64,
) -> Result<UserRow, sqlx::Error> {
    sqlx::query_as::<_, UserRow>(
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
    .bind(id)
    .fetch_one(&mut **tx)
    .await
}

//
// ==============================
// ADMIN
// ==============================
//

/// Marca la cuenta como administrador y fija su contrasena. La usa el arranque para
/// dejar la cuenta del `.env` en el estado que ese fichero declara.
pub async fn promote_admin(
    pool: &PgPool,
    id: i64,
    password_hash: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE users
        SET is_admin = TRUE,
            password = $2
        WHERE id = $1
        "#,
    )
    .bind(id)
    .bind(password_hash)
    .execute(pool)
    .await?;

    Ok(())
}

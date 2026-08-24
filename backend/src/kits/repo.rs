use sqlx::{PgPool, Postgres, Transaction};

use crate::hardware::repo::{PART_COLUMNS, PART_SOURCE};

use super::models::{KitClanRow, KitItemRow, KitRow};

const KIT_COLUMNS: &str = r#"
    k.id, k.name, k.description, k.price, k.scope, k.is_default, k.listed
"#;

/// Las lineas salen en el orden en que se montaria el equipo: primero la placa, porque sin
/// ella no entra nada, y luego el resto. Es el mismo orden que valida `fitting`.
const ITEM_ORDER: &str = r#"
    ORDER BY
        CASE p.kind
            WHEN 'mobo' THEN 0
            WHEN 'cpu'  THEN 1
            WHEN 'ram'  THEN 2
            WHEN 'hdd'  THEN 3
            ELSE 4
        END,
        p.name
"#;

//
// ==============================
// LECTURA
// ==============================
//

/// Todos los kits. Es la vista del panel.
pub async fn find_kits(pool: &PgPool) -> Result<Vec<KitRow>, sqlx::Error> {
    sqlx::query_as::<_, KitRow>(&format!(
        r#"
        SELECT {KIT_COLUMNS}
        FROM hardware_kits k
        ORDER BY k.is_default DESC, k.scope, k.price, k.name
        "#
    ))
    .fetch_all(pool)
    .await
}

/// Los kits que este jugador puede comprar.
///
/// El ambito `marketplace` queda fuera a proposito: existe para el mercado de equipos de
/// NPC, que todavia no esta, y hasta entonces no debe aparecer en la tienda normal.
pub async fn find_offered(pool: &PgPool, user_id: i64) -> Result<Vec<KitRow>, sqlx::Error> {
    sqlx::query_as::<_, KitRow>(&format!(
        r#"
        SELECT {KIT_COLUMNS}
        FROM hardware_kits k
        WHERE k.listed
          AND (
            k.scope = 'shop'
            OR (
              k.scope = 'clan'
              AND EXISTS (
                SELECT 1
                FROM hardware_kit_clans kc
                JOIN clan_members m ON m.clan_id = kc.clan_id
                WHERE kc.kit_id = k.id
                  AND m.user_id = $1
              )
            )
          )
        ORDER BY k.price, k.name
        "#
    ))
    .bind(user_id)
    .fetch_all(pool)
    .await
}

pub async fn find_kit(
    tx: &mut Transaction<'_, Postgres>,
    id: i64,
) -> Result<Option<KitRow>, sqlx::Error> {
    sqlx::query_as::<_, KitRow>(&format!(
        r#"
        SELECT {KIT_COLUMNS}
        FROM hardware_kits k
        WHERE k.id = $1
        "#
    ))
    .bind(id)
    .fetch_optional(&mut **tx)
    .await
}

/// Las lineas de varios kits de una vez. Pintar la lista pide todas: una consulta por kit
/// serian tantas consultas como kits.
pub async fn find_items(pool: &PgPool, kits: &[i64]) -> Result<Vec<KitItemRow>, sqlx::Error> {
    sqlx::query_as::<_, KitItemRow>(&format!(
        r#"
        SELECT ki.kit_id, ki.amount, {PART_COLUMNS}
        FROM hardware_kit_items ki
        JOIN hardware_parts p ON p.id = ki.part_id
        LEFT JOIN hardware_chipsets ch ON ch.id = p.chipset_id
        WHERE ki.kit_id = ANY($1)
        {ITEM_ORDER}
        "#
    ))
    .bind(kits)
    .fetch_all(pool)
    .await
}

/// Lo mismo dentro de una transaccion, para comprar y para validar.
pub async fn lock_items(
    tx: &mut Transaction<'_, Postgres>,
    kit: i64,
) -> Result<Vec<KitItemRow>, sqlx::Error> {
    sqlx::query_as::<_, KitItemRow>(&format!(
        r#"
        SELECT ki.kit_id, ki.amount, {PART_COLUMNS}
        FROM hardware_kit_items ki
        JOIN hardware_parts p ON p.id = ki.part_id
        LEFT JOIN hardware_chipsets ch ON ch.id = p.chipset_id
        WHERE ki.kit_id = $1
        {ITEM_ORDER}
        "#
    ))
    .bind(kit)
    .fetch_all(&mut **tx)
    .await
}

pub async fn find_clans(pool: &PgPool, kits: &[i64]) -> Result<Vec<KitClanRow>, sqlx::Error> {
    sqlx::query_as::<_, KitClanRow>(
        r#"
        SELECT
            kc.kit_id,
            c.id   AS clan_id,
            c.name AS clan_name,
            c.nick AS clan_nick
        FROM hardware_kit_clans kc
        JOIN clans c ON c.id = kc.clan_id
        WHERE kc.kit_id = ANY($1)
        ORDER BY c.name
        "#,
    )
    .bind(kits)
    .fetch_all(pool)
    .await
}

/// Las piezas sueltas que pide un kit, para validarlas antes de guardarlo.
pub async fn find_parts_by_id(
    pool: &PgPool,
    parts: &[i64],
) -> Result<Vec<crate::hardware::models::PartRow>, sqlx::Error> {
    sqlx::query_as::<_, crate::hardware::models::PartRow>(&format!(
        r#"
        SELECT {PART_COLUMNS}
        {PART_SOURCE}
        WHERE p.id = ANY($1)
        "#
    ))
    .bind(parts)
    .fetch_all(pool)
    .await
}

//
// ==============================
// ESCRITURA
// ==============================
//

pub async fn insert_kit(
    tx: &mut Transaction<'_, Postgres>,
    name: &str,
    description: &str,
    price: i64,
    scope: &str,
    listed: bool,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        INSERT INTO hardware_kits(name, description, price, scope, listed)
        VALUES($1, $2, $3, $4, $5)
        RETURNING id
        "#,
    )
    .bind(name)
    .bind(description)
    .bind(price)
    .bind(scope)
    .bind(listed)
    .fetch_one(&mut **tx)
    .await
}

pub async fn update_kit(
    tx: &mut Transaction<'_, Postgres>,
    id: i64,
    name: &str,
    description: &str,
    price: i64,
    scope: &str,
    listed: bool,
) -> Result<u64, sqlx::Error> {
    let done = sqlx::query(
        r#"
        UPDATE hardware_kits
        SET name = $2,
            description = $3,
            price = $4,
            scope = $5,
            listed = $6
        WHERE id = $1
        "#,
    )
    .bind(id)
    .bind(name)
    .bind(description)
    .bind(price)
    .bind(scope)
    .bind(listed)
    .execute(&mut **tx)
    .await?;

    Ok(done.rows_affected())
}

/// El contenido se reemplaza entero en vez de parchearse linea a linea: un kit se valida
/// como conjunto, y editarlo por partes dejaria estados intermedios invalidos guardados.
pub async fn replace_items(
    tx: &mut Transaction<'_, Postgres>,
    kit: i64,
    items: &[(i64, i16)],
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM hardware_kit_items WHERE kit_id = $1")
        .bind(kit)
        .execute(&mut **tx)
        .await?;

    for (part, amount) in items {
        sqlx::query(
            r#"
            INSERT INTO hardware_kit_items(kit_id, part_id, amount)
            VALUES($1, $2, $3)
            "#,
        )
        .bind(kit)
        .bind(part)
        .bind(amount)
        .execute(&mut **tx)
        .await?;
    }

    Ok(())
}

pub async fn replace_clans(
    tx: &mut Transaction<'_, Postgres>,
    kit: i64,
    clans: &[i64],
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM hardware_kit_clans WHERE kit_id = $1")
        .bind(kit)
        .execute(&mut **tx)
        .await?;

    if clans.is_empty() {
        return Ok(());
    }

    sqlx::query(
        r#"
        INSERT INTO hardware_kit_clans(kit_id, clan_id)
        SELECT $1, id
        FROM clans
        WHERE id = ANY($2)
        "#,
    )
    .bind(kit)
    .bind(clans)
    .execute(&mut **tx)
    .await?;

    Ok(())
}

pub async fn delete_kit(pool: &PgPool, id: i64) -> Result<u64, sqlx::Error> {
    let done = sqlx::query("DELETE FROM hardware_kits WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(done.rows_affected())
}

/// Marcar el kit de arranque.
///
/// Primero se apagan todos y luego se enciende el elegido, en la misma transaccion. El
/// indice unico parcial de la tabla haria fallar cualquier otro orden, que es justo lo que
/// se busca: que no exista un instante con dos.
pub async fn set_default(
    tx: &mut Transaction<'_, Postgres>,
    id: i64,
) -> Result<u64, sqlx::Error> {
    sqlx::query("UPDATE hardware_kits SET is_default = FALSE WHERE is_default")
        .execute(&mut **tx)
        .await?;

    let done = sqlx::query("UPDATE hardware_kits SET is_default = TRUE WHERE id = $1")
        .bind(id)
        .execute(&mut **tx)
        .await?;

    Ok(done.rows_affected())
}

pub async fn clear_default(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE hardware_kits SET is_default = FALSE WHERE is_default")
        .execute(pool)
        .await?;

    Ok(())
}

//
// ==============================
// CORPORACIONES
// ==============================
//

/// La lista para el selector del panel.
pub async fn all_clans(pool: &PgPool) -> Result<Vec<KitClanRow>, sqlx::Error> {
    sqlx::query_as::<_, KitClanRow>(
        r#"
        SELECT
            0::BIGINT AS kit_id,
            c.id   AS clan_id,
            c.name AS clan_name,
            c.nick AS clan_nick
        FROM clans c
        ORDER BY c.name
        "#,
    )
    .fetch_all(pool)
    .await
}

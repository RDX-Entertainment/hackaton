use sqlx::{PgPool, Postgres, Transaction};

use super::{
    dtos::PartInput,
    models::{BoardRow, ChipsetRow, ComponentRow, MachineRow, PartRow},
};

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

/// Cuantos equipos tiene. Los de NPC no cuentan: no son suyos.
pub async fn count_machines(
    tx: &mut Transaction<'_, Postgres>,
    user_id: i64,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        SELECT count(*)
        FROM hardware
        WHERE user_id = $1
          AND NOT is_npc
        "#,
    )
    .bind(user_id)
    .fetch_one(&mut **tx)
    .await
}

/// Un equipo vacio. Nace a cero porque la capacidad es la suma de lo instalado, y todavia
/// no lleva nada puesto.
pub async fn insert_machine(
    tx: &mut Transaction<'_, Postgres>,
    user_id: i64,
    name: &str,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        INSERT INTO hardware(user_id, name, cpu, ram, hdd, net)
        VALUES($1, $2, 0, 0, 0, 0)
        RETURNING server_id
        "#,
    )
    .bind(user_id)
    .bind(name)
    .fetch_one(&mut **tx)
    .await
}

pub async fn rename_machine(
    pool: &PgPool,
    machine: i64,
    user_id: i64,
    name: &str,
) -> Result<u64, sqlx::Error> {
    let done = sqlx::query(
        r#"
        UPDATE hardware
        SET name = $3
        WHERE server_id = $1
          AND user_id = $2
          AND NOT is_npc
        "#,
    )
    .bind(machine)
    .bind(user_id)
    .bind(name)
    .execute(pool)
    .await?;

    Ok(done.rows_affected())
}

pub async fn delete_machine(
    tx: &mut Transaction<'_, Postgres>,
    machine: i64,
    user_id: i64,
) -> Result<u64, sqlx::Error> {
    let done = sqlx::query(
        r#"
        DELETE FROM hardware
        WHERE server_id = $1
          AND user_id = $2
          AND NOT is_npc
        "#,
    )
    .bind(machine)
    .bind(user_id)
    .execute(&mut **tx)
    .await?;

    Ok(done.rows_affected())
}

/// Todo lo que hay dentro, placa incluida. Distinto de `count_non_board`, que sirve para
/// decidir si se puede sacar la placa; esto decide si se puede tirar el equipo entero.
pub async fn count_in_machine(
    tx: &mut Transaction<'_, Postgres>,
    machine: i64,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar("SELECT count(*) FROM hardware_components WHERE installed_in = $1")
        .bind(machine)
        .fetch_one(&mut **tx)
        .await
}

/// Fabricar una pieza ya puesta en una maquina. Comprar un kit la usa para entregar el
/// equipo montado en vez de dejarlo en el cajon.
pub async fn insert_installed(
    tx: &mut Transaction<'_, Postgres>,
    part_id: i64,
    user_id: i64,
    machine: i64,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        INSERT INTO hardware_components(part_id, user_id, installed_in)
        VALUES($1, $2, $3)
        RETURNING id
        "#,
    )
    .bind(part_id)
    .bind(user_id)
    .bind(machine)
    .fetch_one(&mut **tx)
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

/// Lo que ya ocupa el disco: quitar una pieza por debajo de esto dejaria ficheros fuera.
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
// PARTS
// ==============================
//

pub const PART_COLUMNS: &str = r#"
    p.id, p.kind, p.name, p.power, p.price, p.generation, p.tier,
    p.cpu_slots, p.ram_slots, p.hdd_slots, p.net_slots,
    p.max_cpu, p.max_ram, p.max_hdd, p.max_net,
    p.chipset_id, ch.name AS chipset, p.listed
"#;

pub const PART_SOURCE: &str = r#"
    FROM hardware_parts p
    LEFT JOIN hardware_chipsets ch ON ch.id = p.chipset_id
"#;

/// El catalogo. `only_listed` es lo que ve la tienda; el panel los ve todos.
pub async fn find_parts(
    pool: &PgPool,
    only_listed: bool,
) -> Result<Vec<PartRow>, sqlx::Error> {
    sqlx::query_as::<_, PartRow>(&format!(
        r#"
        SELECT {PART_COLUMNS}
        {PART_SOURCE}
        WHERE NOT $1 OR p.listed
        ORDER BY p.kind, p.tier, p.power, p.price
        "#
    ))
    .bind(only_listed)
    .fetch_all(pool)
    .await
}

pub async fn find_part(
    tx: &mut Transaction<'_, Postgres>,
    id: i64,
) -> Result<Option<PartRow>, sqlx::Error> {
    sqlx::query_as::<_, PartRow>(&format!(
        r#"
        SELECT {PART_COLUMNS}
        {PART_SOURCE}
        WHERE p.id = $1
        "#
    ))
    .bind(id)
    .fetch_optional(&mut **tx)
    .await
}

pub async fn insert_part(
    pool: &PgPool,
    input: &PartInput,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        INSERT INTO hardware_parts(
            kind, name, power, price, generation, tier, chipset_id,
            cpu_slots, ram_slots, hdd_slots, net_slots,
            max_cpu, max_ram, max_hdd, max_net, listed
        )
        VALUES($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
        RETURNING id
        "#,
    )
    .bind(&input.kind)
    .bind(input.name.trim())
    .bind(input.power)
    .bind(input.price)
    .bind(input.generation)
    .bind(&input.tier)
    .bind(input.chipset_id)
    .bind(input.cpu_slots)
    .bind(input.ram_slots)
    .bind(input.hdd_slots)
    .bind(input.net_slots)
    .bind(input.max_cpu)
    .bind(input.max_ram)
    .bind(input.max_hdd)
    .bind(input.max_net)
    .bind(input.listed)
    .fetch_one(pool)
    .await
}

pub async fn update_part(
    pool: &PgPool,
    id: i64,
    input: &PartInput,
) -> Result<u64, sqlx::Error> {
    let done = sqlx::query(
        r#"
        UPDATE hardware_parts
        SET kind = $2,
            name = $3,
            power = $4,
            price = $5,
            generation = $6,
            tier = $7,
            chipset_id = $8,
            cpu_slots = $9,
            ram_slots = $10,
            hdd_slots = $11,
            net_slots = $12,
            max_cpu = $13,
            max_ram = $14,
            max_hdd = $15,
            max_net = $16,
            listed = $17
        WHERE id = $1
        "#,
    )
    .bind(id)
    .bind(&input.kind)
    .bind(input.name.trim())
    .bind(input.power)
    .bind(input.price)
    .bind(input.generation)
    .bind(&input.tier)
    .bind(input.chipset_id)
    .bind(input.cpu_slots)
    .bind(input.ram_slots)
    .bind(input.hdd_slots)
    .bind(input.net_slots)
    .bind(input.max_cpu)
    .bind(input.max_ram)
    .bind(input.max_hdd)
    .bind(input.max_net)
    .bind(input.listed)
    .execute(pool)
    .await?;

    Ok(done.rows_affected())
}

/// La clave ajena de los componentes es RESTRICT, asi que un part del que ya se fabricaron
/// piezas no se puede borrar. Es deliberado: retirarlo del catalogo es `listed = false`.
pub async fn delete_part(
    pool: &PgPool,
    id: i64,
) -> Result<u64, sqlx::Error> {
    let done = sqlx::query("DELETE FROM hardware_parts WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(done.rows_affected())
}

//
// ==============================
// COMPONENTS
// ==============================
//

const COMPONENT_COLUMNS: &str = r#"
    c.id, c.part_id, c.installed_in,
    k.kind, k.name, k.power, k.generation, k.tier,
    k.cpu_slots, k.ram_slots, k.hdd_slots, k.net_slots,
    k.max_cpu, k.max_ram, k.max_hdd, k.max_net,
    k.chipset_id, ch.name AS chipset
"#;

const COMPONENT_SOURCE: &str = r#"
    FROM hardware_components c
    JOIN hardware_parts k ON k.id = c.part_id
    LEFT JOIN hardware_chipsets ch ON ch.id = k.chipset_id
"#;

/// Todo lo que tiene un jugador: puesto y en el cajon.
pub async fn find_components(
    pool: &PgPool,
    user_id: i64,
) -> Result<Vec<ComponentRow>, sqlx::Error> {
    sqlx::query_as::<_, ComponentRow>(&format!(
        r#"
        SELECT {COMPONENT_COLUMNS}
        {COMPONENT_SOURCE}
        WHERE c.user_id = $1
        ORDER BY k.kind, k.power DESC, c.id
        "#
    ))
    .bind(user_id)
    .fetch_all(pool)
    .await
}

/// Bloquea la fila mientras se decide si la pieza puede moverse.
pub async fn lock_component(
    tx: &mut Transaction<'_, Postgres>,
    id: i64,
    user_id: i64,
) -> Result<Option<ComponentRow>, sqlx::Error> {
    sqlx::query_as::<_, ComponentRow>(&format!(
        r#"
        SELECT {COMPONENT_COLUMNS}
        {COMPONENT_SOURCE}
        WHERE c.id = $1
          AND c.user_id = $2
        FOR UPDATE OF c
        "#
    ))
    .bind(id)
    .bind(user_id)
    .fetch_optional(&mut **tx)
    .await
}

pub async fn insert_component(
    tx: &mut Transaction<'_, Postgres>,
    part_id: i64,
    user_id: i64,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        INSERT INTO hardware_components(part_id, user_id)
        VALUES($1, $2)
        RETURNING id
        "#,
    )
    .bind(part_id)
    .bind(user_id)
    .fetch_one(&mut **tx)
    .await
}

/// Poner en una maquina, o devolver al cajon con `machine = None`.
pub async fn set_installed(
    tx: &mut Transaction<'_, Postgres>,
    id: i64,
    machine: Option<i64>,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE hardware_components SET installed_in = $2 WHERE id = $1")
        .bind(id)
        .bind(machine)
        .execute(&mut **tx)
        .await?;

    Ok(())
}

//
// ==============================
// SLOTS AND CAPACITY
// ==============================
//

pub async fn machine_belongs_to(
    tx: &mut Transaction<'_, Postgres>,
    machine: i64,
    user_id: i64,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        SELECT EXISTS(
            SELECT 1
            FROM hardware
            WHERE server_id = $1
              AND user_id = $2
              AND NOT is_npc
        )
        "#,
    )
    .bind(machine)
    .bind(user_id)
    .fetch_one(&mut **tx)
    .await
}

/// La placa puesta en una maquina, que es quien decide que cabe dentro.
pub async fn find_board(
    tx: &mut Transaction<'_, Postgres>,
    machine: i64,
) -> Result<Option<BoardRow>, sqlx::Error> {
    sqlx::query_as::<_, BoardRow>(
        r#"
        SELECT
            k.generation,
            k.tier,
            k.chipset_id,
            k.cpu_slots,
            k.ram_slots,
            k.hdd_slots,
            k.net_slots,
            k.max_cpu,
            k.max_ram,
            k.max_hdd,
            k.max_net
        FROM hardware_components c
        JOIN hardware_parts k ON k.id = c.part_id
        WHERE c.installed_in = $1
          AND k.kind = 'mobo'
        LIMIT 1
        "#,
    )
    .bind(machine)
    .fetch_optional(&mut **tx)
    .await
}

pub async fn count_installed(
    tx: &mut Transaction<'_, Postgres>,
    machine: i64,
    kind: &str,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        SELECT count(*)
        FROM hardware_components c
        JOIN hardware_parts k ON k.id = c.part_id
        WHERE c.installed_in = $1
          AND k.kind = $2
        "#,
    )
    .bind(machine)
    .bind(kind)
    .fetch_one(&mut **tx)
    .await
}

/// Piezas puestas en la maquina que no son la placa. Sacar la placa con algo dentro
/// dejaria componentes sin ranura que los sostenga.
pub async fn count_non_board(
    tx: &mut Transaction<'_, Postgres>,
    machine: i64,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        SELECT count(*)
        FROM hardware_components c
        JOIN hardware_parts k ON k.id = c.part_id
        WHERE c.installed_in = $1
          AND k.kind <> 'mobo'
        "#,
    )
    .bind(machine)
    .fetch_one(&mut **tx)
    .await
}

/// Cuanto suma lo ya instalado de un tipo en una maquina. Es lo que se compara con el
/// techo que impone la placa.
pub async fn installed_power(
    tx: &mut Transaction<'_, Postgres>,
    machine: i64,
    kind: &str,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        SELECT COALESCE(sum(k.power), 0)::bigint
        FROM hardware_components c
        JOIN hardware_parts k ON k.id = c.part_id
        WHERE c.installed_in = $1
          AND k.kind = $2
        "#,
    )
    .bind(machine)
    .bind(kind)
    .fetch_one(&mut **tx)
    .await
}

/// Capacidad total del jugador para un tipo, sumando todas sus maquinas. Es el numero que
/// el resto del juego lee, asi que es el que hay que comparar con lo que esta en uso.
pub async fn total_capacity(
    tx: &mut Transaction<'_, Postgres>,
    user_id: i64,
    kind: &str,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        SELECT COALESCE(sum(k.power), 0)::bigint
        FROM hardware_components c
        JOIN hardware_parts k ON k.id = c.part_id
        WHERE c.user_id = $1
          AND c.installed_in IS NOT NULL
          AND k.kind = $2
        "#,
    )
    .bind(user_id)
    .bind(kind)
    .fetch_one(&mut **tx)
    .await
}

/// Rehace las cuatro columnas desde lo que hay puesto. Es la unica escritura de capacidad
/// que queda en el sistema: cualquier movimiento de pieza termina aqui.
pub async fn recompute_machine(
    tx: &mut Transaction<'_, Postgres>,
    machine: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE hardware h
        SET cpu = COALESCE(s.cpu, 0),
            ram = COALESCE(s.ram, 0),
            hdd = COALESCE(s.hdd, 0),
            net = COALESCE(s.net, 0)
        FROM (
            SELECT
                sum(k.power) FILTER (WHERE k.kind = 'cpu') AS cpu,
                sum(k.power) FILTER (WHERE k.kind = 'ram') AS ram,
                sum(k.power) FILTER (WHERE k.kind = 'hdd') AS hdd,
                sum(k.power) FILTER (WHERE k.kind = 'net') AS net
            FROM hardware_components c
            JOIN hardware_parts k ON k.id = c.part_id
            WHERE c.installed_in = $1
        ) s
        WHERE h.server_id = $1
        "#,
    )
    .bind(machine)
    .execute(&mut **tx)
    .await?;

    Ok(())
}

//
// ==============================
// ADMIN
// ==============================
//

pub async fn user_exists(
    pool: &PgPool,
    user_id: i64,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE id = $1)")
        .bind(user_id)
        .fetch_one(pool)
        .await
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

//
// ==============================
// CHIPSETS
// ==============================
//

pub async fn find_chipsets(
    pool: &PgPool,
) -> Result<Vec<ChipsetRow>, sqlx::Error> {
    sqlx::query_as::<_, ChipsetRow>(
        r#"
        SELECT id, name
        FROM hardware_chipsets
        ORDER BY name
        "#,
    )
    .fetch_all(pool)
    .await
}

/// Cuantas piezas del catalogo dependen de este chipset.
pub async fn chipset_usage(
    pool: &PgPool,
    id: i64,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar("SELECT count(*) FROM hardware_parts WHERE chipset_id = $1")
        .bind(id)
        .fetch_one(pool)
        .await
}

pub async fn delete_chipset(
    pool: &PgPool,
    id: i64,
) -> Result<u64, sqlx::Error> {
    let done = sqlx::query("DELETE FROM hardware_chipsets WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(done.rows_affected())
}

pub async fn insert_chipset(
    pool: &PgPool,
    name: &str,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        INSERT INTO hardware_chipsets(name)
        VALUES($1)
        RETURNING id
        "#,
    )
    .bind(name)
    .fetch_one(pool)
    .await
}

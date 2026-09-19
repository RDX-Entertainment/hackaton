use sqlx::{PgPool, Postgres, Transaction};

pub async fn require_computer(tx: &mut Transaction<'_, Postgres>, machine: i64) -> Result<(), crate::error::ApiError> {
    let kind: Option<String> = sqlx::query_scalar("SELECT node_kind FROM hardware WHERE server_id=$1 FOR UPDATE")
        .bind(machine).fetch_optional(&mut **tx).await?;
    if kind.as_deref() != Some("computer") {
        return Err(crate::error::ApiError::BadRequest("Un switch no admite componentes ni tareas de ordenador".into()));
    }
    Ok(())
}

use super::{
    dtos::PartInput,
    models::{BoardRow, ChipsetRow, ComponentRow, LinkRow, MachineRow, PartRow},
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
            is_gateway, node_kind, switch_ports, operational, disabled_reasons,
            server_id,
            name,
            cpu,
            hdd,
            ram,
            net,
            map_x,
            map_y,
            public_ip,
            lan_ip
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
          AND node_kind = 'computer'
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
        INSERT INTO hardware(user_id, name, cpu, ram, hdd, net, lan_ip)
        VALUES(
            $1, $2, 0, 0, 0, 0,
            -- La siguiente libre de su red, empezando por 10.0.0.1. Se calcula aqui y no
            -- fuera porque dentro del INSERT va en la misma transaccion que la crea: dos
            -- altas a la vez no pueden mirar el mismo hueco y quedarse con el.
            (SELECT (10::bigint << 24) | (coalesce(max(lan_ip & 255), 0) + 1)
             FROM hardware
             WHERE user_id = $1 AND NOT is_npc)
        )
        RETURNING server_id
        "#,
    )
    .bind(user_id)
    .bind(name)
    .fetch_one(&mut **tx)
    .await
}

pub async fn insert_npc_machine(
    tx: &mut Transaction<'_, Postgres>,
    npc_id: i64,
    name: &str,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        INSERT INTO hardware(user_id, name, cpu, ram, hdd, net, is_npc)
        VALUES($1, $2, 0, 0, 0, 0, TRUE)
        RETURNING server_id
        "#,
    )
    .bind(npc_id)
    .bind(name)
    .fetch_one(&mut **tx)
    .await
}

/// NPCs have at most one hardware record. Existing NPCs imported into a network may not
/// have one yet, so the organization editor can create it lazily.
pub async fn find_npc_machine(tx: &mut Transaction<'_, Postgres>, npc_id: i64) -> Result<Option<i64>, sqlx::Error> {
    sqlx::query_scalar("SELECT server_id FROM hardware WHERE user_id = $1 AND is_npc LIMIT 1")
        .bind(npc_id).fetch_optional(&mut **tx).await
}

pub async fn clear_machine_components(tx: &mut Transaction<'_, Postgres>, machine: i64) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM hardware_components WHERE installed_in = $1")
        .bind(machine).execute(&mut **tx).await?;
    Ok(())
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

pub async fn insert_npc_installed(
    tx: &mut Transaction<'_, Postgres>,
    part_id: i64,
    npc_id: i64,
    machine: i64,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        INSERT INTO hardware_components(part_id, user_id, is_npc, installed_in)
        VALUES($1, $2, TRUE, $3)
        RETURNING id
        "#,
    )
    .bind(part_id)
    .bind(npc_id)
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
          AND NOT c.is_npc
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
          AND NOT c.is_npc
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


/// Rehace las cuatro columnas desde lo que hay puesto. Es la unica escritura de capacidad
/// que queda en el sistema: cualquier movimiento de pieza termina aqui.
pub async fn recompute_machine(
    tx: &mut Transaction<'_, Postgres>,
    machine: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        SELECT refresh_hardware($1)
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

//
// ==============================
// MAPA DE NODOS
// ==============================
//

/// Mover un equipo por el mapa. Devuelve cuantas filas cambiaron: 0 significa que ese
/// equipo no es de quien lo pide, y el servicio lo traduce a un error.
pub async fn move_machine(
    pool: &PgPool,
    machine: i64,
    user_id: i64,
    map_x: i32,
    map_y: i32,
) -> Result<u64, sqlx::Error> {
    let done = sqlx::query(
        r#"
        UPDATE hardware
        SET map_x = $3,
            map_y = $4
        WHERE server_id = $1
          AND user_id = $2
          AND NOT is_npc
        "#,
    )
    .bind(machine)
    .bind(user_id)
    .bind(map_x)
    .bind(map_y)
    .execute(pool)
    .await?;

    Ok(done.rows_affected())
}

//
// ==============================
// CLUSTERS
// ==============================
//

/// Los equipos que cuelgan del controlador **siguiendo cables**, sin salir de su dueno.
///
/// Un cluster se monta sobre el cable y no al reves: lo que no esta conectado no se agrupa. La
/// consulta recorre la red desde el controlador y devuelve todo lo que se alcanza por cable,
/// que es el conjunto entre el que se puede elegir. No hace falta que los miembros esten
/// cableados **al** controlador directamente; basta que se llegue.
pub async fn cabled_component(
    tx: &mut Transaction<'_, Postgres>,
    controller: i64,
) -> Result<Vec<i64>, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        WITH RECURSIVE alcanzables(server_id) AS (
            SELECT $1::bigint

            UNION

            SELECT CASE
                       WHEN l.from_server = a.server_id THEN l.to_server
                       ELSE l.from_server
                   END
            FROM alcanzables a
            JOIN hardware_links l
              ON l.from_server = a.server_id
              OR l.to_server   = a.server_id
        )
        SELECT server_id FROM alcanzables
        "#,
    )
    .bind(controller)
    .fetch_all(&mut **tx)
    .await
}

pub async fn insert_cluster(
    tx: &mut Transaction<'_, Postgres>,
    user_id: i64,
    name: &str,
    controller: i64,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        "INSERT INTO hardware_clusters(user_id, name, controller_server)          VALUES($1, $2, $3) RETURNING id",
    )
    .bind(user_id)
    .bind(name)
    .bind(controller)
    .fetch_one(&mut **tx)
    .await
}

pub async fn join_cluster(
    tx: &mut Transaction<'_, Postgres>,
    cluster: i64,
    machines: &[i64],
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE hardware SET cluster_id = $1 WHERE server_id = ANY($2)")
        .bind(cluster)
        .bind(machines)
        .execute(&mut **tx)
        .await?;

    Ok(())
}

/// Deshacer un grupo. Los equipos vuelven a ser lo que eran, con lo suyo.
pub async fn dissolve_cluster(
    tx: &mut Transaction<'_, Postgres>,
    cluster: i64,
    user_id: i64,
) -> Result<bool, sqlx::Error> {
    sqlx::query("UPDATE hardware SET cluster_id = NULL WHERE cluster_id = $1")
        .bind(cluster)
        .execute(&mut **tx)
        .await?;

    let borrados = sqlx::query("DELETE FROM hardware_clusters WHERE id = $1 AND user_id = $2")
        .bind(cluster)
        .bind(user_id)
        .execute(&mut **tx)
        .await?
        .rows_affected();

    Ok(borrados > 0)
}

/// Si un equipo ya esta en algun grupo. Un equipo pertenece a uno como mucho.
pub async fn cluster_of(
    tx: &mut Transaction<'_, Postgres>,
    machine: i64,
) -> Result<Option<i64>, sqlx::Error> {
    sqlx::query_scalar("SELECT cluster_id FROM hardware WHERE server_id = $1")
        .bind(machine)
        .fetch_optional(&mut **tx)
        .await
        .map(Option::flatten)
}

pub async fn find_links(
    pool: &PgPool,
    user_id: i64,
) -> Result<Vec<LinkRow>, sqlx::Error> {
    sqlx::query_as::<_, LinkRow>(
        r#"
        SELECT id, from_server, to_server
        FROM hardware_links
        WHERE user_id = $1
        ORDER BY id
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

/// Si ese equipo es suyo. Se comprueban los dos extremos antes de cablear: sin esto,
/// bastaria mandar el id de la maquina de otro para colgarse de su red.
pub async fn owns_machine(
    pool: &PgPool,
    machine: i64,
    user_id: i64,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)
        FROM hardware
        WHERE server_id = $1
          AND user_id = $2
          AND NOT is_npc
        "#,
    )
    .bind(machine)
    .bind(user_id)
    .fetch_one(pool)
    .await
    .map(|found| found > 0)
}

/// Cuantas bocas de red da la placa que tiene puesta ahora mismo. Es el mismo dato que
/// `lan_total` en las redes de organizacion, y sale del mismo sitio: sin placa no hay
/// puertos, y cambiar de placa cambia el limite.
pub async fn lan_ports(
    tx: &mut Transaction<'_, Postgres>,
    machine: i64,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar::<_, Option<i64>>(
        r#"
        SELECT CASE WHEN h.node_kind='switch' THEN h.switch_ports ELSE
          (SELECT p.net_slots FROM hardware_components c JOIN hardware_parts p ON p.id=c.part_id
           WHERE c.installed_in=h.server_id AND p.kind='mobo' LIMIT 1) END::bigint
        FROM hardware h WHERE h.server_id=$1
        "#,
    )
    .bind(machine)
    .fetch_optional(&mut **tx)
    .await
    .map(|found| found.flatten().unwrap_or(0))
}

/// Cables que ya salen de ese equipo, mirando por los dos extremos.
pub async fn count_links_of(
    tx: &mut Transaction<'_, Postgres>,
    machine: i64,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        SELECT count(*)
        FROM hardware_links
        WHERE from_server = $1
           OR to_server = $1
        "#,
    )
    .bind(machine)
    .fetch_one(&mut **tx)
    .await
}

pub async fn insert_link(
    tx: &mut Transaction<'_, Postgres>,
    user_id: i64,
    from: i64,
    to: i64,
) -> Result<LinkRow, sqlx::Error> {
    sqlx::query_as::<_, LinkRow>(
        r#"
        INSERT INTO hardware_links (user_id, from_server, to_server)
        VALUES ($1, $2, $3)
        RETURNING id, from_server, to_server
        "#,
    )
    .bind(user_id)
    .bind(from)
    .bind(to)
    .fetch_one(&mut **tx)
    .await
}

pub async fn delete_link(
    pool: &PgPool,
    link: i64,
    user_id: i64,
) -> Result<u64, sqlx::Error> {
    let done = sqlx::query(
        r#"
        DELETE FROM hardware_links
        WHERE id = $1
          AND user_id = $2
        "#,
    )
    .bind(link)
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(done.rows_affected())
}

/// Si el equipo tiene tarjeta de red puesta, que es lo que le da salida a Internet.
///
/// Esa salida ocupa boca igual que un cable a otro equipo: es la misma regla que aplica el
/// mapa de organizaciones, donde un nodo publico cuenta `INTERNET` entre sus conexiones.
pub async fn has_uplink(
    tx: &mut Transaction<'_, Postgres>,
    machine: i64,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)
        FROM hardware_components c
        JOIN hardware_parts p ON p.id = c.part_id
        WHERE c.installed_in = $1
          AND p.kind = 'net'
        "#,
    )
    .bind(machine)
    .fetch_one(&mut **tx)
    .await
    .map(|found| found > 0)
}

//
// ==============================
// DIRECCIONES DE LA MAQUINA
// ==============================
//

/// Cuantas tarjetas de red lleva puestas. Es lo que decide si tiene salida a Internet, y
/// por tanto si le toca direccion publica.
pub async fn count_nics(
    tx: &mut Transaction<'_, Postgres>,
    machine: i64,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        SELECT count(*)
        FROM hardware_components c
        JOIN hardware_parts p ON p.id = c.part_id
        WHERE c.installed_in = $1
          AND p.kind = 'net'
        "#,
    )
    .bind(machine)
    .fetch_one(&mut **tx)
    .await
}

/// Darle direccion publica, si no tenia. Sortearla dentro de la transaccion es lo que
/// impide que dos instalaciones a la vez se lleven la misma: `random_free_ip()` ya mira la
/// tabla, y el indice unico remata lo que se le escape.
/// Cuantas plazas de flota tiene un jugador.
///
/// Empieza en una y sube pagando. Es el techo **del jugador**; el del servidor vive en el
/// entorno y lo lee `server_slot_ceiling`.
pub async fn machine_slots(
    tx: &mut Transaction<'_, Postgres>,
    user_id: i64,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar("SELECT machine_slots::bigint FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(&mut **tx)
        .await
        .map(|found| found.unwrap_or(1))
}

/// Una plaza mas. Se llama despues de cobrarla, dentro de la misma transaccion.
pub async fn add_machine_slot(
    tx: &mut Transaction<'_, Postgres>,
    user_id: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE users SET machine_slots = machine_slots + 1 WHERE id = $1")
        .bind(user_id)
        .execute(&mut **tx)
        .await?;

    Ok(())
}

pub async fn assign_public_ip(
    tx: &mut Transaction<'_, Postgres>,
    machine: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE hardware
        SET public_ip = random_free_ip()
        WHERE server_id = $1
          AND public_ip IS NULL
        "#,
    )
    .bind(machine)
    .execute(&mut **tx)
    .await?;

    Ok(())
}

/// Quitarsela. La direccion vuelve al bombo: quien saca la tarjeta deja de estar en
/// Internet, y guardarle el numero seria reservar algo que ya no usa.
/// Si un equipo es la maquina principal de su dueno.
///
/// Se pregunta antes de dejarlo sin salida: la principal lleva la direccion con la que el
/// jugador aparece en todas partes, asi que apagarla no es lo mismo que apagar cualquier otra.
pub async fn is_gateway(
    tx: &mut Transaction<'_, Postgres>,
    machine: i64,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar("SELECT is_gateway FROM hardware WHERE server_id = $1")
        .bind(machine)
        .fetch_optional(&mut **tx)
        .await
        .map(|found| found.unwrap_or(false))
}

/// Serialize principal changes with component removal/moves for this owner.
pub async fn lock_hardware_owner(tx: &mut Transaction<'_, Postgres>, user: i64) -> Result<(), sqlx::Error> {
    sqlx::query("SELECT id FROM users WHERE id=$1 FOR UPDATE")
        .bind(user).fetch_one(&mut **tx).await?;
    Ok(())
}

pub async fn protect_gateway_uplink(tx: &mut Transaction<'_, Postgres>, machine: i64) -> Result<(), crate::error::ApiError> {
    if is_gateway(tx, machine).await? && count_nics(tx, machine).await? == 0 {
        return Err(crate::error::ApiError::BadRequest(
            "GATEWAY_NEEDS_NIC: esta es tu máquina principal. Nombra principal a otra desde Hardware antes de dejar esta sin salida".into(),
        ));
    }
    Ok(())
}

pub async fn clear_public_ip(
    tx: &mut Transaction<'_, Postgres>,
    machine: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE hardware SET public_ip = NULL WHERE server_id = $1")
        .bind(machine)
        .execute(&mut **tx)
        .await?;

    Ok(())
}

/// Cuanto suma un tipo de pieza **en una maquina**, no en toda la flota.
///
/// El agregado por dueno sigue existiendo para la memoria, que todavia es un deposito comun;
/// el disco ya no, desde que el software vive en un equipo concreto.
pub async fn machine_capacity(
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

/// Lo que ocupa el software guardado en esa maquina.
pub async fn machine_disk_in_use(
    tx: &mut Transaction<'_, Postgres>,
    machine: i64,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        SELECT COALESCE(sum(soft_size), 0)::bigint
        FROM software
        WHERE server_id = $1
        "#,
    )
    .bind(machine)
    .fetch_one(&mut **tx)
    .await
}

/// La memoria que ocupa lo que corre **en esa maquina**.
pub async fn machine_memory_in_use(
    tx: &mut Transaction<'_, Postgres>,
    machine: i64,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        SELECT COALESCE(sum(s.soft_ram), 0)::bigint
        FROM software s
        JOIN software_running r ON r.soft_id = s.id
        WHERE s.server_id = $1
        "#,
    )
    .bind(machine)
    .fetch_one(&mut **tx)
    .await
}

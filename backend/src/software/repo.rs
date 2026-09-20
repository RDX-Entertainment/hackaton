use sqlx::{PgPool, Postgres, Transaction};

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
            s.parent_id,
            (r.soft_id IS NOT NULL) AS running,
            s.server_id
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

/// El disco de **una** maquina, no la suma de las del dueno.
///
/// Antes esto sumaba todas: meter un disco en cualquier caja daba sitio a todo el software
/// del jugador, estuviera donde estuviera. Con el software viviendo en un equipo concreto, lo
/// que decide si algo cabe es el disco de ese equipo y solo ese.
pub async fn machine_hdd(
    pool: &PgPool,
    machine: i64,
    user_id: i64,
) -> Result<Option<f32>, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        SELECT hdd::real
        FROM hardware
        WHERE server_id = $1
          AND user_id = $2
          AND NOT is_npc
        "#,
    )
    .bind(machine)
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map(Option::flatten)
}

/// Lo que ocupa el software guardado en esa maquina.
pub async fn used_hdd(
    pool: &PgPool,
    machine: i64,
) -> Result<Option<i64>, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        SELECT SUM(soft_size)::bigint
        FROM software
        WHERE server_id = $1
        "#,
    )
    .bind(machine)
    .fetch_one(pool)
    .await
}

pub async fn create_software(
    pool: &PgPool,
    user_id: i64,
    machine: i64,
    name: &str,
    size: i32,
    ram: i32,
    soft_type: i16,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        INSERT INTO software(
            user_id,
            server_id,
            soft_name,
            soft_version,
            soft_size,
            soft_ram,
            soft_type
        )
        VALUES($1, $2, $3, 1, $4, $5, $6)
        RETURNING id
        "#,
    )
    .bind(user_id)
    .bind(machine)
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
    server: Option<i64>,
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
            is_folder,
            parent_id
        FROM software
        WHERE user_id = $1
          AND is_npc = $2
          AND NOT soft_hidden
          AND server_id = $3
        ORDER BY soft_name
        "#,
    )
    .bind(owner_id)
    .bind(is_npc)
    .bind(server)
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
    victim_server: Option<i64>,
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
            -- A que maquina va. `victim_id` dice de quien es; esto, a cual de las suyas.
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
            $7
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
    .bind(victim_server)
    .fetch_one(pool)
    .await
}

pub async fn create_delete_process(
    pool: &PgPool,
    user_id: i64,
    victim_id: i64,
    victim_server: Option<i64>,
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
            -- A que maquina va. `victim_id` dice de quien es; esto, a cual de las suyas.
            victim_server
        )
        VALUES(
            $1,
            $2,
            3,
            $3,
            $4,
            $4,
            now() + ($5::integer * interval '1 second'),
            $5,
            (SELECT server_id FROM software WHERE id = $3),
            $6
        )
        RETURNING pid
        "#,
    )
    .bind(user_id)
    .bind(victim_id)
    .bind(software_id)
    .bind(info)
    .bind(seconds)
    .bind(victim_server)
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
            time_ideal,
            -- Nace donde vive su programa, y se escribe ahora: mover el software despues no
            -- reescribe de donde salio esto.
            origin_server
        )
        VALUES(
            $1,
            0,
            17,
            $2,
            'local',
            'local',
            now() + ($3::integer * interval '1 second'),
            $3,
            (SELECT server_id FROM software WHERE id = $2)
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

pub async fn user_exists(pool: &PgPool, user_id: i64) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE id = $1)")
        .bind(user_id)
        .fetch_one(pool)
        .await
}

/// Un fichero regalado por un administrador cae en la maquina principal de su dueno.
///
/// Devuelve `None` cuando ese jugador no tiene ninguna: desde la 0047 un programa vive en un
/// disco, y no hay disco donde ponerlo. Antes esto insertaba sin decir la maquina y reventaba
/// contra el `NOT NULL`.
pub async fn create_admin_software(
    pool: &PgPool,
    user_id: i64,
    name: &str,
    version: i32,
    size: i32,
    ram: i32,
    soft_type: i16,
) -> Result<Option<i64>, sqlx::Error> {
    let mut tx = pool.begin().await?;
    let id = sqlx::query_scalar(
        r#"
        INSERT INTO software(user_id,soft_name,soft_version,soft_size,soft_ram,soft_type,server_id)
        SELECT $1,$2,$3,$4,$5,$6,h.server_id
          FROM hardware h
         WHERE h.user_id = $1 AND NOT h.is_npc AND h.is_gateway
         ORDER BY h.server_id
         LIMIT 1
        RETURNING id
        "#,
    )
    .bind(user_id)
    .bind(name)
    .bind(version)
    .bind(size)
    .bind(ram)
    .bind(soft_type)
    .fetch_optional(&mut *tx)
    .await?;
    if let Some(id) = id {
        upsert_admin_software_profile(&mut tx, id, soft_type, size, ram).await?;
    }
    tx.commit().await?;
    Ok(id)
}

/// El editor administrativo crea programas heredados, pero siguen necesitando el perfil
/// modular que relaciona el tipo clásico con una capacidad. Sin esta fila un fichero tipo
/// SSH existe en el disco, aunque el arsenal no puede reconocerlo como `ssh_exploit`.
async fn upsert_admin_software_profile(
    tx: &mut Transaction<'_, Postgres>,
    software_id: i64,
    soft_type: i16,
    size: i32,
    ram: i32,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO software_profiles(
            software_id, origin, capability_id, cpu_need, ram_need, net_need, weight
        )
        SELECT $1, 'legacy', c.id, GREATEST(1,$4), GREATEST(1,$4), 1, GREATEST(1,$3)
          FROM (VALUES (1,'cracker'), (2,'anti_bruteforce'), (3,'port_scan'),
                       (4,'firewall_ssh'), (7,'antivirus'), (13,'ftp_exploit'),
                       (14,'ssh_exploit'), (99,'virus_infect')) AS legacy(soft_type, capability)
          JOIN software_capabilities c ON c.code=legacy.capability
         WHERE legacy.soft_type=$2
        ON CONFLICT(software_id) DO UPDATE SET
            capability_id=EXCLUDED.capability_id,
            cpu_need=EXCLUDED.cpu_need,
            ram_need=EXCLUDED.ram_need,
            weight=EXCLUDED.weight
        "#,
    )
    .bind(software_id)
    .bind(soft_type)
    .bind(size)
    .bind(ram)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

pub async fn update_admin_software(
    pool: &PgPool,
    user_id: i64,
    software_id: i64,
    name: &str,
    version: i32,
    size: i32,
    ram: i32,
    soft_type: i16,
) -> Result<bool, sqlx::Error> {
    let mut tx = pool.begin().await?;
    let updated = sqlx::query(
        "UPDATE software SET soft_name=$3,soft_version=$4,soft_size=$5,soft_ram=$6,soft_type=$7 WHERE id=$1 AND user_id=$2 AND NOT is_npc",
    )
    .bind(software_id)
    .bind(user_id)
    .bind(name)
    .bind(version)
    .bind(size)
    .bind(ram)
    .bind(soft_type)
    .execute(&mut *tx)
    .await?
    .rows_affected() > 0;
    if updated {
        upsert_admin_software_profile(&mut tx, software_id, soft_type, size, ram).await?;
    }
    tx.commit().await?;
    Ok(updated)
}

pub async fn delete_admin_software(pool: &PgPool, user_id: i64, software_id: i64) -> Result<bool, sqlx::Error> {
    sqlx::query("DELETE FROM software_running WHERE soft_id=$1")
        .bind(software_id)
        .execute(pool)
        .await?;
    Ok(sqlx::query("DELETE FROM software WHERE id=$1 AND user_id=$2 AND NOT is_npc")
        .bind(software_id)
        .bind(user_id)
        .execute(pool)
        .await?
        .rows_affected() > 0)
}

pub async fn set_admin_software_running(
    pool: &PgPool,
    user_id: i64,
    software_id: i64,
    running: bool,
) -> Result<bool, sqlx::Error> {
    let owned: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM software WHERE id=$1 AND user_id=$2 AND NOT is_npc)")
        .bind(software_id)
        .bind(user_id)
        .fetch_one(pool)
        .await?;
    if !owned { return Ok(false); }
    if running {
        sqlx::query("INSERT INTO software_running(soft_id,user_id,is_npc) VALUES($1,$2,false) ON CONFLICT(soft_id) DO UPDATE SET user_id=EXCLUDED.user_id,is_npc=false")
            .bind(software_id).bind(user_id).execute(pool).await?;
    } else {
        sqlx::query("DELETE FROM software_running WHERE soft_id=$1").bind(software_id).execute(pool).await?;
    }
    Ok(true)
}

//
// ==============================
// MOVER ENTRE MAQUINAS
// ==============================
//

/// Un fichero propio, con lo que hace falta para decidir si se puede mover: donde esta,
/// cuanto ocupa y si esta cargado.
pub async fn find_owned_software(
    pool: &PgPool,
    software: i64,
    user_id: i64,
) -> Result<Option<(i64, i32, bool)>, sqlx::Error> {
    sqlx::query_as::<_, (i64, i32, bool)>(
        r#"
        SELECT
            s.server_id,
            s.soft_size,
            (r.soft_id IS NOT NULL) AS running
        FROM software s
        LEFT JOIN software_running r ON r.soft_id = s.id
        WHERE s.id = $1
          AND s.user_id = $2
          AND NOT s.is_npc
        "#,
    )
    .bind(software)
    .bind(user_id)
    .fetch_optional(pool)
    .await
}

/// Si hay cable entre esas dos, mirado por los dos extremos.
pub async fn are_linked(
    pool: &PgPool,
    a: i64,
    b: i64,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)
        FROM hardware_links
        WHERE (from_server = $1 AND to_server = $2)
           OR (from_server = $2 AND to_server = $1)
        "#,
    )
    .bind(a)
    .bind(b)
    .fetch_one(pool)
    .await
    .map(|found| found > 0)
}

/// Si la maquina sale a Internet. Dos que salen pueden pasarse un fichero por ahi aunque no
/// compartan cable.
pub async fn has_public_ip(
    pool: &PgPool,
    machine: i64,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar::<_, Option<i64>>(
        "SELECT public_ip FROM hardware WHERE server_id = $1",
    )
    .bind(machine)
    .fetch_optional(pool)
    .await
    .map(|found| found.flatten().is_some())
}

pub async fn set_software_machine(
    pool: &PgPool,
    software: i64,
    machine: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE software SET server_id = $2 WHERE id = $1")
        .bind(software)
        .bind(machine)
        .execute(pool)
        .await?;

    Ok(())
}

//
// ==============================
// CARPETAS
// ==============================
//

/// Una carpeta no ocupa disco ni memoria: es un sitio, no un fichero.
pub async fn insert_folder(
    pool: &PgPool,
    user_id: i64,
    server_id: i64,
    parent_id: Option<i64>,
    name: &str,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        "INSERT INTO software
            (user_id, soft_name, soft_version, soft_size, soft_ram, soft_type,
             is_folder, is_npc, server_id, parent_id)
         VALUES ($1, $2, 0, 0, 0, 0, TRUE, FALSE, $3, $4)
         RETURNING id",
    )
    .bind(user_id)
    .bind(name)
    .bind(server_id)
    .bind(parent_id)
    .fetch_one(pool)
    .await
}

/// Lo que hace falta saber de una entrada antes de moverla o borrarla.
pub async fn find_entry(
    pool: &PgPool,
    id: i64,
    user_id: i64,
) -> Result<Option<(i64, bool, bool)>, sqlx::Error> {
    sqlx::query_as::<_, (i64, bool, bool)>(
        "SELECT s.server_id, s.is_folder, (r.soft_id IS NOT NULL)
         FROM software s
         LEFT JOIN software_running r ON r.soft_id = s.id
         WHERE s.id = $1 AND s.user_id = $2 AND NOT s.is_npc",
    )
    .bind(id)
    .bind(user_id)
    .fetch_optional(pool)
    .await
}

/// Que la carpeta de destino exista, sea carpeta, sea suya y este en esa maquina.
pub async fn folder_here(
    pool: &PgPool,
    id: i64,
    user_id: i64,
    server_id: i64,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT EXISTS(
            SELECT 1 FROM software
            WHERE id = $1 AND user_id = $2 AND server_id = $3
              AND is_folder AND NOT is_npc)",
    )
    .bind(id)
    .bind(user_id)
    .bind(server_id)
    .fetch_one(pool)
    .await
}

/// Si `candidata` esta dentro de `raiz`, a cualquier profundidad.
///
/// Es lo que impide meter una carpeta dentro de si misma o de una hija suya, que
/// dejaria un trozo del arbol colgando fuera de la raiz y sin forma de volver.
pub async fn is_inside(
    pool: &PgPool,
    candidata: i64,
    raiz: i64,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar(
        "WITH RECURSIVE arriba AS (
            SELECT id, parent_id FROM software WHERE id = $1
            UNION ALL
            SELECT s.id, s.parent_id FROM software s
            JOIN arriba a ON s.id = a.parent_id
         )
         SELECT EXISTS(SELECT 1 FROM arriba WHERE id = $2)",
    )
    .bind(candidata)
    .bind(raiz)
    .fetch_one(pool)
    .await
}

pub async fn folder_is_empty(pool: &PgPool, id: i64) -> Result<bool, sqlx::Error> {
    let hay: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM software WHERE parent_id = $1)")
        .bind(id)
        .fetch_one(pool)
        .await?;

    Ok(!hay)
}

pub async fn set_parent(
    pool: &PgPool,
    id: i64,
    parent_id: Option<i64>,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE software SET parent_id = $2 WHERE id = $1")
        .bind(id)
        .bind(parent_id)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn delete_entry(pool: &PgPool, id: i64, user_id: i64) -> Result<u64, sqlx::Error> {
    let hecho = sqlx::query("DELETE FROM software WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(user_id)
        .execute(pool)
        .await?;

    Ok(hecho.rows_affected())
}

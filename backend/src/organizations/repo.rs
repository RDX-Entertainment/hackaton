use sqlx::{PgPool, Postgres, Transaction};

use super::models::{BehaviorRow, NetworkNpcRow, NetworkRow, NodeLinkRow, NpcCatalogRow, OrganizationRow};

pub async fn find_all(pool: &PgPool) -> Result<Vec<OrganizationRow>, sqlx::Error> {
    sqlx::query_as::<_, OrganizationRow>(
        r#"
        SELECT id, slug, name, category, description, active, created_at, updated_at
        FROM organizations
        ORDER BY name
        "#,
    )
    .fetch_all(pool)
    .await
}

pub async fn find_by_id(pool: &PgPool, id: i64) -> Result<Option<OrganizationRow>, sqlx::Error> {
    sqlx::query_as::<_, OrganizationRow>(
        r#"
        SELECT id, slug, name, category, description, active, created_at, updated_at
        FROM organizations
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn find_behaviors(
    pool: &PgPool,
    organizations: &[i64],
) -> Result<Vec<BehaviorRow>, sqlx::Error> {
    sqlx::query_as::<_, BehaviorRow>(
        r#"
        SELECT organization_id, code, config::text AS config
        FROM organization_behaviors
        WHERE organization_id = ANY($1)
        ORDER BY code
        "#,
    )
    .bind(organizations)
    .fetch_all(pool)
    .await
}

pub async fn find_networks(
    pool: &PgPool,
    organizations: &[i64],
) -> Result<Vec<NetworkRow>, sqlx::Error> {
    sqlx::query_as::<_, NetworkRow>(
        r#"
        SELECT id, organization_id, name, zone, created_at
        FROM organization_networks
        WHERE organization_id = ANY($1)
        ORDER BY name
        "#,
    )
    .bind(organizations)
    .fetch_all(pool)
    .await
}

pub async fn find_network_npcs(
    pool: &PgPool,
    networks: &[i64],
) -> Result<Vec<NetworkNpcRow>, sqlx::Error> {
    sqlx::query_as::<_, NetworkNpcRow>(
        r#"
        SELECT
            m.network_id,
            n.id AS npc_id,
            n.npc_ip,
            n.npc_type,
            COALESCE(i.name, '') AS npc_name,
            m.role,
            m.source_kit_id,
            -- Publico e interno no se guardan: los dice la interfaz de red instalada, y esa
            -- lectura vive una sola vez en `npc_internet_exposure`.
            CASE WHEN e.is_public THEN 'public' ELSE 'internal' END AS zone,
            m.map_x,
            m.map_y,
            -- Ranuras de la placa menos los cables ya tendidos, menos la que gasta la salida
            -- a Internet cuando el nodo es publico.
            mobo.net_slots
              - (SELECT count(*) FROM organization_node_links l WHERE n.id IN (l.from_npc_id, l.to_npc_id))
              - CASE WHEN e.is_public THEN 1 ELSE 0 END AS lan_available,
            mobo.net_slots::bigint AS lan_total,
            COALESCE((SELECT count(*) FROM hardware_components c JOIN hardware h ON h.server_id = c.installed_in WHERE h.is_npc AND h.user_id = n.id), 0) AS computer_components
        FROM organization_network_npcs m
        JOIN npc n ON n.id = m.npc_id
        JOIN npc_internet_exposure e ON e.npc_id = n.id
        LEFT JOIN npc_info i ON i.npc_id = n.id
        CROSS JOIN LATERAL (
            SELECT COALESCE((
                SELECT p.net_slots
                FROM hardware h
                JOIN hardware_components c ON c.installed_in = h.server_id
                JOIN hardware_parts p ON p.id = c.part_id
                WHERE h.is_npc AND h.user_id = n.id AND p.kind = 'mobo'
                LIMIT 1
            ), 0) AS net_slots
        ) AS mobo
        WHERE m.network_id = ANY($1)
        ORDER BY n.npc_ip
        "#,
    )
    .bind(networks)
    .fetch_all(pool)
    .await
}

pub async fn find_npc_catalog(pool: &PgPool) -> Result<Vec<NpcCatalogRow>, sqlx::Error> {
    sqlx::query_as::<_, NpcCatalogRow>(
        r#"
        SELECT
            n.id,
            n.npc_ip AS ip,
            n.npc_type,
            COALESCE(i.name, '') AS name,
            net.organization_id,
            m.network_id,
            net.zone
        FROM npc n
        LEFT JOIN npc_info i ON i.npc_id = n.id
        LEFT JOIN organization_network_npcs m ON m.npc_id = n.id
        LEFT JOIN organization_networks net ON net.id = m.network_id
        ORDER BY n.npc_ip
        "#,
    )
    .fetch_all(pool)
    .await
}

pub async fn find_links(pool: &PgPool, organizations: &[i64]) -> Result<Vec<NodeLinkRow>, sqlx::Error> {
    sqlx::query_as("SELECT id, organization_id, from_npc_id, to_npc_id FROM organization_node_links WHERE organization_id = ANY($1)").bind(organizations).fetch_all(pool).await
}
pub async fn insert_link(pool: &PgPool, organization: i64, first: i64, second: i64) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar("INSERT INTO organization_node_links(organization_id, from_npc_id, to_npc_id) VALUES($1, $2, $3) ON CONFLICT(organization_id, from_npc_id, to_npc_id) DO UPDATE SET from_npc_id = EXCLUDED.from_npc_id RETURNING id").bind(organization).bind(first.min(second)).bind(first.max(second)).fetch_one(pool).await
}
pub async fn available_lan_ports(pool: &PgPool, organization: i64, npc: i64) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        r#"SELECT COALESCE((SELECT p.net_slots FROM hardware h JOIN hardware_components c ON c.installed_in = h.server_id JOIN hardware_parts p ON p.id = c.part_id WHERE h.is_npc AND h.user_id = $2 AND p.kind = 'mobo' LIMIT 1), 0)
          - (SELECT count(*) FROM organization_node_links l WHERE l.organization_id = $1 AND ($2 IN (l.from_npc_id, l.to_npc_id)))
          - CASE WHEN EXISTS (SELECT 1 FROM hardware_components c JOIN hardware h ON h.server_id = c.installed_in JOIN hardware_parts p ON p.id = c.part_id WHERE h.is_npc AND h.user_id = $2 AND p.kind = 'net') THEN 1 ELSE 0 END
          "#,
    ).bind(organization).bind(npc).fetch_one(pool).await
}
pub async fn delete_link(pool: &PgPool, organization: i64, id: i64) -> Result<u64, sqlx::Error> { Ok(sqlx::query("DELETE FROM organization_node_links WHERE id = $1 AND organization_id = $2").bind(id).bind(organization).execute(pool).await?.rows_affected()) }

pub async fn insert(
    tx: &mut Transaction<'_, Postgres>,
    slug: &str,
    name: &str,
    category: &str,
    description: &str,
    active: bool,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        INSERT INTO organizations(slug, name, category, description, active)
        VALUES($1, $2, $3, $4, $5)
        RETURNING id
        "#,
    )
    .bind(slug)
    .bind(name)
    .bind(category)
    .bind(description)
    .bind(active)
    .fetch_one(&mut **tx)
    .await
}

pub async fn update(
    tx: &mut Transaction<'_, Postgres>,
    id: i64,
    slug: &str,
    name: &str,
    category: &str,
    description: &str,
    active: bool,
) -> Result<u64, sqlx::Error> {
    let done = sqlx::query(
        r#"
        UPDATE organizations
        SET slug = $2,
            name = $3,
            category = $4,
            description = $5,
            active = $6,
            updated_at = now()
        WHERE id = $1
        "#,
    )
    .bind(id)
    .bind(slug)
    .bind(name)
    .bind(category)
    .bind(description)
    .bind(active)
    .execute(&mut **tx)
    .await?;

    Ok(done.rows_affected())
}

/// Deja los comportamientos que pide la lista, y solo toca lo que cambia.
///
/// Antes borraba todos y los reinsertaba. Eso dejo de valer en cuanto `banking` gano una
/// clave ajena desde `bank_accounts`: el borrado saltaba el RESTRICT aunque el mismo
/// comportamiento volviera a entrar una linea despues, y una organizacion que fuera banco con
/// cuentas abiertas no se podia ni renombrar. Borrar solo lo que sobra deja el RESTRICT para
/// cuando de verdad se le quita el `banking` a un banco en uso, que es lo que debe proteger.
pub async fn replace_behaviors(
    tx: &mut Transaction<'_, Postgres>,
    organization: i64,
    behaviors: &[(String, String)],
) -> Result<(), sqlx::Error> {
    let codes: Vec<String> =
        behaviors.iter().map(|(code, _)| code.clone()).collect();

    sqlx::query(
        r#"
        DELETE FROM organization_behaviors
        WHERE organization_id = $1
          AND code <> ALL($2)
        "#,
    )
    .bind(organization)
    .bind(&codes)
    .execute(&mut **tx)
    .await?;

    for (code, config) in behaviors {
        sqlx::query(
            r#"
            INSERT INTO organization_behaviors(organization_id, code, config)
            VALUES($1, $2, $3::jsonb)
            ON CONFLICT(organization_id, code)
            DO UPDATE SET config = EXCLUDED.config
            "#,
        )
        .bind(organization)
        .bind(code)
        .bind(config)
        .execute(&mut **tx)
        .await?;
    }

    Ok(())
}

pub async fn delete(pool: &PgPool, id: i64) -> Result<u64, sqlx::Error> {
    let done = sqlx::query("DELETE FROM organizations WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(done.rows_affected())
}

pub async fn insert_network(
    pool: &PgPool,
    organization: i64,
    name: &str,
    zone: &str,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        INSERT INTO organization_networks(organization_id, name, zone)
        VALUES($1, $2, $3)
        RETURNING id
        "#,
    )
    .bind(organization)
    .bind(name)
    .bind(zone)
    .fetch_one(pool)
    .await
}

pub async fn update_network(
    pool: &PgPool,
    organization: i64,
    id: i64,
    name: &str,
    zone: &str,
) -> Result<u64, sqlx::Error> {
    let done = sqlx::query(
        r#"
        UPDATE organization_networks
        SET name = $3, zone = $4
        WHERE id = $1 AND organization_id = $2
        "#,
    )
    .bind(id)
    .bind(organization)
    .bind(name)
    .bind(zone)
    .execute(pool)
    .await?;

    Ok(done.rows_affected())
}

pub async fn delete_network(pool: &PgPool, organization: i64, id: i64) -> Result<u64, sqlx::Error> {
    let done =
        sqlx::query("DELETE FROM organization_networks WHERE id = $1 AND organization_id = $2")
            .bind(id)
            .bind(organization)
            .execute(pool)
            .await?;

    Ok(done.rows_affected())
}

pub async fn network_belongs_to(
    pool: &PgPool,
    organization: i64,
    network: i64,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM organization_networks WHERE id = $1 AND organization_id = $2)",
    )
    .bind(network)
    .bind(organization)
    .fetch_one(pool)
    .await
}

pub async fn npc_exists(pool: &PgPool, npc: i64) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM npc WHERE id = $1)")
        .bind(npc)
        .fetch_one(pool)
        .await
}

/// Mover es un upsert: una IP no puede estar a la vez en dos segmentos, y no hay un
/// instante entre quitarla de uno y ponerla en otro en el que vuelva a ser publica.
pub async fn assign_npc(
    pool: &PgPool,
    network: i64,
    npc: i64,
    role: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO organization_network_npcs(network_id, npc_id, role)
        VALUES($1, $2, $3)
        ON CONFLICT(npc_id)
        DO UPDATE SET network_id = EXCLUDED.network_id, role = EXCLUDED.role
        "#,
    )
    .bind(network)
    .bind(npc)
    .bind(role)
    .execute(pool)
    .await?;

    Ok(())
}

/// La zona no se escribe: la decide el hardware del nodo. Aqui solo viajan el sitio en el
/// mapa y el rol, que son las dos cosas que el administrador si elige a mano.
pub async fn update_node(
    pool: &PgPool, organization: i64, network: i64, npc: i64, map_x: i32, map_y: i32, role: &str,
) -> Result<u64, sqlx::Error> {
    let done = sqlx::query(
        r#"UPDATE organization_network_npcs AS m
           SET map_x = $4, map_y = $5, role = $6
           FROM organization_networks AS n
           WHERE m.network_id = $2 AND m.npc_id = $3 AND n.id = m.network_id AND n.organization_id = $1"#,
    )
    .bind(organization).bind(network).bind(npc).bind(map_x).bind(map_y).bind(role)
    .execute(pool).await?;
    Ok(done.rows_affected())
}

pub async fn node_belongs_to(pool: &PgPool, organization: i64, network: i64, npc: i64) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM organization_network_npcs m JOIN organization_networks n ON n.id = m.network_id WHERE n.organization_id = $1 AND m.network_id = $2 AND m.npc_id = $3)")
        .bind(organization).bind(network).bind(npc).fetch_one(pool).await
}

/// Corta todos los cables que llegan a este nodo. Va antes de sacarlo de la red: un enlace a un
/// nodo que ya no esta en el mapa se dibujaria contra la nada.
pub async fn delete_node_links(tx: &mut Transaction<'_, Postgres>, organization: i64, npc: i64) -> Result<u64, sqlx::Error> {
    let done = sqlx::query("DELETE FROM organization_node_links WHERE organization_id = $1 AND (from_npc_id = $2 OR to_npc_id = $2)")
        .bind(organization).bind(npc).execute(&mut **tx).await?;
    Ok(done.rows_affected())
}

/// Saca el nodo de su red. El NPC sobrevive a proposito: pudo entrar desde el catalogo con
/// `assign_npc`, y borrarlo alli destruiria una ficha que este panel no creo.
pub async fn delete_node(tx: &mut Transaction<'_, Postgres>, organization: i64, network: i64, npc: i64) -> Result<u64, sqlx::Error> {
    let done = sqlx::query(
        r#"DELETE FROM organization_network_npcs AS m
           USING organization_networks AS n
           WHERE m.network_id = $2 AND m.npc_id = $3 AND n.id = m.network_id AND n.organization_id = $1"#,
    )
    .bind(organization).bind(network).bind(npc)
    .execute(&mut **tx).await?;
    Ok(done.rows_affected())
}

pub async fn set_node_source_kit(tx: &mut Transaction<'_, Postgres>, network: i64, npc: i64, kit: Option<i64>) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE organization_network_npcs SET source_kit_id = $3 WHERE network_id = $1 AND npc_id = $2")
        .bind(network).bind(npc).bind(kit).execute(&mut **tx).await?;
    Ok(())
}

pub async fn computer_configuration(pool: &PgPool, network: i64, npc: i64) -> Result<(Option<i64>, Vec<(i64, i16)>), sqlx::Error> {
    let kit = sqlx::query_scalar("SELECT source_kit_id FROM organization_network_npcs WHERE network_id = $1 AND npc_id = $2").bind(network).bind(npc).fetch_optional(pool).await?.flatten();
    let items = sqlx::query_as::<_, (i64, i16)>("SELECT c.part_id, count(*)::smallint FROM hardware_components c JOIN hardware h ON h.server_id = c.installed_in WHERE h.is_npc AND h.user_id = $1 GROUP BY c.part_id ORDER BY c.part_id").bind(npc).fetch_all(pool).await?;
    Ok((kit, items))
}



pub async fn insert_npc(
    tx: &mut Transaction<'_, Postgres>,
    npc_type: i16,
    ip: i64,
    password: &str,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        INSERT INTO npc(npc_type, npc_ip, npc_pass)
        VALUES($1, $2, $3)
        RETURNING id
        "#,
    )
    .bind(npc_type)
    .bind(ip)
    .bind(password)
    .fetch_one(&mut **tx)
    .await
}

pub async fn insert_npc_info(
    tx: &mut Transaction<'_, Postgres>,
    npc: i64,
    name: &str,
    web: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO npc_info(npc_id, name, web) VALUES($1, $2, $3)")
        .bind(npc)
        .bind(name)
        .bind(web)
        .execute(&mut **tx)
        .await?;
    Ok(())
}

pub async fn insert_npc_log(
    tx: &mut Transaction<'_, Postgres>,
    npc: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO log(user_id, text, is_npc) VALUES($1, '', TRUE)")
        .bind(npc)
        .execute(&mut **tx)
        .await?;
    Ok(())
}

pub async fn assign_npc_with_kit(
    tx: &mut Transaction<'_, Postgres>,
    network: i64,
    npc: i64,
    role: &str,
    kit: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO organization_network_npcs(network_id, npc_id, role, source_kit_id)
        VALUES($1, $2, $3, $4)
        "#,
    )
    .bind(network)
    .bind(npc)
    .bind(role)
    .bind(kit)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

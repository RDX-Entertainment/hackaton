use sqlx::{
    PgPool,
    Postgres,
    Transaction,
};

use super::models::{
    BankOfMachineRow,
    AnalyzeResultRow,
    BestAnalyzerRow,
    BookmarkRow,
    HistoryRow,
    InternetTargetRow,
    LanRemoteRow,
    NpcRemoteRow,
    SiteRow,
    PlayerRemoteRow,
};

//
// ==============================
// TARGETS
// ==============================
//

/// El listin de quien pregunta.
///
/// Lleva un `viewer` desde que hay NPC que no son del mundo abierto: el campo de tiro de cada
/// jugador son maquinas suyas, y meterlas en el listin comun lo llenaria de objetivos que a
/// los demas no les sirven de nada. `npc_visible_to` guarda esa regla, y la guarda tambien
/// para `find_npc_by_ip`: quien no ve una maquina en la lista tampoco la encuentra
/// escribiendo su direccion.
pub async fn targets(
    pool: &PgPool,
    viewer: i64,
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
        JOIN npc_internet_exposure e
            ON e.npc_id = n.id
        WHERE n.down_until <= now()
          -- Los NPCs antiguos sin infraestructura siguen publicos. Uno asignado solo
          -- aparece si tiene interfaz de red y su organizacion esta activa. Quien lo
          -- decide es la vista: la regla vive en un sitio, no repartida por consultas.
          AND e.reachable
          AND npc_visible_to(n.id, $1)
        ORDER BY
            n.npc_type,
            n.npc_ip
        LIMIT 200
        "#,
    )
    .bind(viewer)
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
    viewer: i64,
) -> Result<Option<NpcRemoteRow>, sqlx::Error> {
    sqlx::query_as::<_, NpcRemoteRow>(
        r#"
        SELECT
            n.id,
            n.npc_type,
            n.down_until > now() OR NOT coalesce(h.operational, false) AS down,
            COALESCE(i.name, '') AS name,
            i.web,
            h.server_id
        FROM npc n
        LEFT JOIN npc_info i
            ON i.npc_id = n.id

        -- La maquina que responde. Desde la 0049 lleva puesta la `npc_ip`, asi que se une por
        -- ella y no por el dueno: es la direccion la que identifica el nodo.
        LEFT JOIN hardware h
            ON h.public_ip = n.npc_ip
           AND h.is_npc
        JOIN npc_internet_exposure e
            ON e.npc_id = n.id
        WHERE n.npc_ip = $1
          -- Hasta que exista pivoting, las redes internas son invisibles incluso si se
          -- conoce la IP. El motor de rutas sustituira esta condicion mas adelante.
          -- Quitarle la interfaz de red a un nodo lo apaga de Internet aqui mismo: su
          -- web deja de servirse porque su IP deja de resolver.
          AND e.reachable
          -- El campo de tiro de otro no responde ni sabiendo su direccion. Es la misma regla
          -- que el listin, y por eso es la misma funcion.
          AND npc_visible_to(n.id, $2)
        "#,
    )
    .bind(ip)
    .bind(viewer)
    .fetch_optional(pool)
    .await
}

/// La maquina que hay en una direccion **de red interna**, si el que pregunta puede llegar.
///
/// Una `lan_ip` no esta en Internet: no la resuelve nadie desde fuera, y por eso no vale con
/// conocerla. Hace falta **estar dentro** — tener acceso vivo a una maquina que tenga un cable
/// hasta ella. Eso es lo que convierte el mapa de nodos en mecanica en vez de dibujo, y lo que
/// le da sentido a esconder un equipo dejandolo sin tarjeta de red: sin salida propia, la unica
/// forma de tocarlo es entrar antes por otro.
///
/// El acceso vale tanto si se obtuvo por la direccion publica del pivote como por la interna,
/// asi que las cadenas funcionan solas: de la primera maquina a la segunda, y de la segunda a
/// la tercera.
///
/// Una `lan_ip` solo es unica dentro de la red de su dueno —el indice es `(user_id, lan_ip)`—,
/// asi que dos jugadores pueden tener la misma. No es ambiguo: lo que la resuelve no es el
/// numero sino desde donde se pregunta, y el cable pertenece a un solo dueno.
pub async fn find_machine_on_lan(
    pool: &PgPool,
    ip: i64,
    viewer: i64,
) -> Result<Option<LanRemoteRow>, sqlx::Error> {
    sqlx::query_as::<_, LanRemoteRow>(
        r#"
        WITH RECURSIVE reachable(origin, node) AS (
          SELECT h.server_id, h.server_id FROM hardware h
          WHERE h.operational AND (h.user_id=$2 AND NOT h.is_npc OR EXISTS(
            SELECT 1 FROM internet_access a WHERE a.user_id=$2 AND a.ip IN(h.public_ip,h.lan_ip)))
          UNION
          SELECT r.origin, peer.server_id FROM reachable r
          JOIN hardware hop ON hop.server_id=r.node
          JOIN hardware_links cable ON cable.from_server=r.node OR cable.to_server=r.node
          JOIN hardware peer ON peer.server_id=CASE WHEN cable.from_server=r.node THEN cable.to_server ELSE cable.from_server END
          WHERE (r.node=r.origin OR hop.node_kind='switch') AND hop.operational
            AND peer.user_id=hop.user_id AND peer.is_npc=hop.is_npc
        )
        SELECT
            NOT destino.operational AS down,
            destino.server_id,
            destino.user_id AS owner_id,
            destino.is_npc,
            COALESCE(u.login, i.name, destino.name) AS name,
            COALESCE(p.web, i.web) AS web

        FROM hardware destino

        JOIN reachable r ON r.node=destino.server_id AND r.origin<>destino.server_id

        LEFT JOIN users u
            ON u.id = destino.user_id AND NOT destino.is_npc
        LEFT JOIN player_profiles p
            ON p.user_id = destino.user_id AND NOT destino.is_npc
        LEFT JOIN npc_info i
            ON i.npc_id = destino.user_id AND destino.is_npc

        WHERE destino.lan_ip = $1
          AND destino.node_kind='computer'

        ORDER BY destino.server_id
        LIMIT 1
        "#,
    )
    .bind(ip)
    .bind(viewer)
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
            NOT h.operational AS down,
            u.id,
            u.login,
            p.web,
            h.server_id
        FROM hardware h

        JOIN users u
            ON u.id = h.user_id

        LEFT JOIN player_profiles p
            ON p.user_id = u.id

        WHERE h.public_ip = $1
          AND NOT h.is_npc
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

/// The compromised list remembers which account was obtained through an attack. A normal
/// password login has no row there, so callers can fall back to `root` in that case.
pub async fn access_login(
    pool: &PgPool,
    user_id: i64,
    ip: i64,
) -> Result<Option<String>, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT login FROM compromised_lists WHERE user_id = $1 AND ip = $2 ORDER BY hacked_at DESC NULLS LAST LIMIT 1",
    )
    .bind(user_id)
    .bind(ip)
    .fetch_optional(pool)
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

/// La llave de una maquina.
///
/// Sustituye a `npc_password` y `player_password`, que preguntaban por el **dueno**: acertar
/// la contrasena en cualquiera de sus direcciones abria todas las demas, y separar direcciones
/// no servia de nada si la cerradura seguia siendo una. Los NPC ya lo tenian bien —cada uno
/// trae la suya— y eran los jugadores los que arrastraban el modelo de una caja por cuenta.
///
/// `None` si la direccion no tiene maquina detras, que desde la 0049 no deberia pasar con
/// nada alcanzable.
pub async fn machine_password(
    pool: &PgPool,
    machine: i64,
) -> Result<Option<String>, sqlx::Error> {
    sqlx::query_scalar("SELECT machine_pass FROM hardware WHERE server_id = $1")
        .bind(machine)
        .fetch_optional(pool)
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

/// El campo de tiro pertenece al alumno. Ni otro NPC ni el de otro jugador da esta excepcion.
pub async fn is_own_practice(pool: &PgPool, user_id: i64, npc_id: i64) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM npc_purpose
         WHERE npc_id = $1 AND kind = 'practice' AND owner_id = $2)",
    )
    .bind(npc_id)
    .bind(user_id)
    .fetch_one(pool)
    .await
}

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
    victim_server: Option<i64>,
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
            time_ideal,
            -- A que maquina va. `victim_id` sigue diciendo de quien es, pero desde que un
            -- jugador puede tener varias eso ya no dice a cual se ataca, y el crackeo tiene
            -- que revelar la llave de la que se ataco.
            victim_server
        )
        VALUES(
            $1,
            $2,
            $3,
            $4,
            $4,
            now() + ($5::integer * interval '1 second'),
            $5,
            $6
        )
        RETURNING pid
        "#,
    )
    .bind(user_id)
    .bind(victim_id)
    .bind(action)
    .bind(info)
    .bind(seconds)
    .bind(victim_server)
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
    victim_server: Option<i64>,
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
            time_ideal,
            -- A que maquina va. `victim_id` dice de quien es; esto, a cual de las suyas.
            victim_server
        )
        VALUES(
            $1,
            $2,
            8,
            $3,
            $3,
            now() + ($4::integer * interval '1 second'),
            $4,
            $5
        )
        RETURNING pid
        "#,
    )
    .bind(user_id)
    .bind(victim_id)
    .bind(info)
    .bind(seconds)
    .bind(victim_server)
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
            -- Y a que maquina va. `victim_id` dice de quien es; esto, a cual de las suyas.
            victim_server
        )
        VALUES(
            $1,
            $2,
            23,
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
//
// ==============================
// SITIOS
// ==============================
//

/// Todas las paginas que hay, sirvalas quien las sirva. El panel las edita igual.
pub async fn all_sites(pool: &PgPool) -> Result<Vec<SiteRow>, sqlx::Error> {
    sqlx::query_as::<_, SiteRow>(
        r#"
        SELECT
            'npc'                  AS kind,
            n.id,
            n.npc_ip               AS ip,
            COALESCE(i.name, '')   AS name,
            COALESCE(i.web, '')    AS web
        FROM npc n
        LEFT JOIN npc_info i ON i.npc_id = n.id

        UNION ALL

        SELECT
            'player'               AS kind,
            u.id,
            u.game_ip              AS ip,
            u.login                AS name,
            COALESCE(p.web, '')    AS web
        FROM users u
        LEFT JOIN player_profiles p ON p.user_id = u.id

        ORDER BY kind, name
        "#,
    )
    .fetch_all(pool)
    .await
}

pub async fn set_npc_site(pool: &PgPool, id: i64, web: &str) -> Result<u64, sqlx::Error> {
    let done = sqlx::query("UPDATE npc_info SET web = $2 WHERE npc_id = $1")
        .bind(id)
        .bind(web)
        .execute(pool)
        .await?;

    Ok(done.rows_affected())
}

pub async fn set_player_site(pool: &PgPool, id: i64, web: &str) -> Result<u64, sqlx::Error> {
    let done = sqlx::query("UPDATE player_profiles SET web = $2, updated_at = now() WHERE user_id = $1")
        .bind(id)
        .bind(web)
        .execute(pool)
        .await?;

    Ok(done.rows_affected())
}

/// Si esta maquina es de un banco, cual. Un banco es una organizacion activa con el
/// comportamiento `banking`, la misma condicion que hace cumplir `bank_accounts`.
pub async fn bank_of_npc(
    pool: &PgPool,
    npc_id: i64,
) -> Result<Option<BankOfMachineRow>, sqlx::Error> {
    sqlx::query_as::<_, BankOfMachineRow>(
        r#"
        SELECT o.id, o.name
        FROM organization_network_npcs m
        JOIN organization_networks net
            ON net.id = m.network_id
        JOIN organizations o
            ON o.id = net.organization_id
        JOIN organization_behaviors b
            ON b.organization_id = o.id
           AND b.code = 'banking'
        WHERE m.npc_id = $1
          AND o.active
        "#,
    )
    .bind(npc_id)
    .fetch_optional(pool)
    .await
}

/// Que familias de ordenes puede usar la web de esta maquina.
///
/// Los cuatro alcances se resuelven de una vez: `all_sites` vale siempre;
/// `all_organizations` en cuanto la maquina pertenezca a alguna; `organization` y `site`
/// cuando el destino coincide. Un `action` de `*` concede la familia entera, y uno concreto
/// solo ese verbo -que se filtra despues, ya con el catalogo delante-.
pub async fn granted_actions(
    pool: &PgPool,
    npc_id: i64,
) -> Result<Vec<(String, String)>, sqlx::Error> {
    sqlx::query_as::<_, (String, String)>(
        r#"
        SELECT DISTINCT g.family, g.action
        FROM action_grants g
        LEFT JOIN organization_network_npcs m
            ON m.npc_id = $1
        LEFT JOIN organization_networks net
            ON net.id = m.network_id
        WHERE g.scope = 'all_sites'
           OR (g.scope = 'all_organizations' AND m.npc_id IS NOT NULL)
           OR (g.scope = 'organization' AND g.organization_id = net.organization_id)
           OR (g.scope = 'site' AND g.npc_id = $1)
        "#,
    )
    .bind(npc_id)
    .fetch_all(pool)
    .await
}

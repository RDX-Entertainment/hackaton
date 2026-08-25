use std::collections::{HashMap, HashSet};

use rand::{Rng, distr::Alphanumeric};
use sqlx::PgPool;

use crate::{
    auth::Ctx,
    error::ApiError,
    hardware::{fitting, repo as hardware},
    internet::sanitize,
    kits::repo as kits,
};

use super::{
    dtos::{
        BehaviorInput, BehaviorResponse, ComputerBuildInput, ComputerBuildItemInput, ComputerConfigurationResponse, CreateServerInput, NetworkInput, NetworkNpcInput, NodeInput,
        NetworkNpcResponse, NetworkResponse, NodeLinkInput, NodeLinkResponse, NpcCatalogResponse, OrganizationInput,
        OrganizationResponse, ServerCreatedResponse,
    },
    models::OrganizationRow,
    repo,
};

const MAX_DESCRIPTION: usize = 4_000;
const ZONES: [&str; 4] = ["public", "dmz", "internal", "restricted"];

fn is_code(value: &str, max: usize) -> bool {
    !value.is_empty()
        && value.len() <= max
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

fn validate(input: &OrganizationInput) -> Result<Vec<(String, String)>, ApiError> {
    let slug = input.slug.trim();
    let category = input.category.trim();

    if !is_code(slug, 64) {
        return Err(ApiError::BadRequest(
            "slug must use lowercase letters, numbers, and hyphens".into(),
        ));
    }

    if input.name.trim().is_empty() || input.name.trim().len() > 80 {
        return Err(ApiError::BadRequest(
            "name must contain 1 to 80 characters".into(),
        ));
    }

    if !is_code(category, 40) {
        return Err(ApiError::BadRequest(
            "category must use lowercase letters, numbers, and hyphens".into(),
        ));
    }

    if input.description.trim().len() > MAX_DESCRIPTION {
        return Err(ApiError::BadRequest(format!(
            "description is too long; maximum is {MAX_DESCRIPTION} characters"
        )));
    }

    let mut codes = HashSet::new();
    let mut result = Vec::with_capacity(input.behaviors.len());

    for BehaviorInput { code, config } in &input.behaviors {
        let code = code.trim();

        if !is_code(code, 60) {
            return Err(ApiError::BadRequest(
                "behavior code must use lowercase letters, numbers, and hyphens".into(),
            ));
        }

        if !config.is_object() {
            return Err(ApiError::BadRequest(
                "behavior config must be a JSON object".into(),
            ));
        }

        if !codes.insert(code) {
            return Err(ApiError::BadRequest(format!(
                "DUPLICATE_BEHAVIOR: {code} is repeated"
            )));
        }

        result.push((code.to_owned(), config.to_string()));
    }

    Ok(result)
}

async fn fill(
    pool: &PgPool,
    rows: Vec<OrganizationRow>,
) -> Result<Vec<OrganizationResponse>, ApiError> {
    if rows.is_empty() {
        return Ok(Vec::new());
    }

    let ids: Vec<i64> = rows.iter().map(|row| row.id).collect();
    let mut by_organization: HashMap<i64, Vec<BehaviorResponse>> = HashMap::new();
    let networks = repo::find_networks(pool, &ids).await?;
    let network_ids: Vec<i64> = networks.iter().map(|network| network.id).collect();
    let mut npcs_by_network: HashMap<i64, Vec<NetworkNpcResponse>> = HashMap::new();
    let mut networks_by_organization: HashMap<i64, Vec<NetworkResponse>> = HashMap::new();
    let mut links_by_organization: HashMap<i64, Vec<NodeLinkResponse>> = HashMap::new();
    for link in repo::find_links(pool, &ids).await? { links_by_organization.entry(link.organization_id).or_default().push(NodeLinkResponse::from(link)); }

    for behavior in repo::find_behaviors(pool, &ids).await? {
        let organization = behavior.organization_id;
        by_organization
            .entry(organization)
            .or_default()
            .push(BehaviorResponse::from(behavior));
    }

    if !network_ids.is_empty() {
        for npc in repo::find_network_npcs(pool, &network_ids).await? {
            let network = npc.network_id;
            npcs_by_network
                .entry(network)
                .or_default()
                .push(NetworkNpcResponse::from(npc));
        }
    }

    for network in networks {
        let organization = network.organization_id;
        let npcs = npcs_by_network.remove(&network.id).unwrap_or_default();
        networks_by_organization
            .entry(organization)
            .or_default()
            .push(NetworkResponse::new(network, npcs));
    }

    Ok(rows
        .into_iter()
        .map(|row| {
            let organization = row.id;
            let behaviors = by_organization.remove(&row.id).unwrap_or_default();
            let networks = networks_by_organization.remove(&row.id).unwrap_or_default();
            OrganizationResponse::new(row, behaviors, networks, links_by_organization.remove(&organization).unwrap_or_default())
        })
        .collect())
}

fn validate_network(input: &NetworkInput) -> Result<(), ApiError> {
    if input.name.trim().is_empty() || input.name.trim().len() > 60 {
        return Err(ApiError::BadRequest(
            "network name must contain 1 to 60 characters".into(),
        ));
    }

    if !ZONES.contains(&input.zone.as_str()) {
        return Err(ApiError::BadRequest(format!(
            "zone must be one of {}",
            ZONES.join(", ")
        )));
    }

    Ok(())
}

fn validate_network_npc(input: &NetworkNpcInput) -> Result<(), ApiError> {
    if input.npc_id <= 0 {
        return Err(ApiError::BadRequest("npc_id must be positive".into()));
    }

    if !is_code(input.role.trim(), 40) {
        return Err(ApiError::BadRequest(
            "role must use lowercase letters, numbers, and hyphens".into(),
        ));
    }

    Ok(())
}

fn validate_server(input: &CreateServerInput) -> Result<(), ApiError> {
    if input.name.trim().is_empty() || input.name.trim().len() > 50 {
        return Err(ApiError::BadRequest(
            "server name must contain 1 to 50 characters".into(),
        ));
    }

    if input.kit_id <= 0 {
        return Err(ApiError::BadRequest("kit_id must be positive".into()));
    }

    validate_network_npc(&NetworkNpcInput {
        npc_id: 1,
        role: input.role.clone(),
    })?;

    if input.npc_type < 0 {
        return Err(ApiError::BadRequest("npc_type must be non-negative".into()));
    }

    sanitize::check(&input.web)?;
    Ok(())
}

pub async fn list(pool: &PgPool, ctx: Ctx) -> Result<Vec<OrganizationResponse>, ApiError> {
    ctx.require_admin()?;
    fill(pool, repo::find_all(pool).await?).await
}

pub async fn get(pool: &PgPool, ctx: Ctx, id: i64) -> Result<OrganizationResponse, ApiError> {
    ctx.require_admin()?;
    let row = repo::find_by_id(pool, id)
        .await?
        .ok_or(ApiError::NotFound)?;

    fill(pool, vec![row])
        .await?
        .into_iter()
        .next()
        .ok_or(ApiError::NotFound)
}

pub async fn create(pool: &PgPool, ctx: Ctx, input: OrganizationInput) -> Result<i64, ApiError> {
    ctx.require_admin()?;
    let behaviors = validate(&input)?;

    let mut tx = pool.begin().await?;
    let id = repo::insert(
        &mut tx,
        input.slug.trim(),
        input.name.trim(),
        input.category.trim(),
        input.description.trim(),
        input.active,
    )
    .await
    .map_err(|error| unique_error(error, "slug or name"))?;

    repo::replace_behaviors(&mut tx, id, &behaviors)
        .await
        .map_err(behaviors_in_use)?;
    tx.commit().await?;

    Ok(id)
}

pub async fn edit(
    pool: &PgPool,
    ctx: Ctx,
    id: i64,
    input: OrganizationInput,
) -> Result<(), ApiError> {
    ctx.require_admin()?;
    let behaviors = validate(&input)?;
    let mut tx = pool.begin().await?;

    if repo::update(
        &mut tx,
        id,
        input.slug.trim(),
        input.name.trim(),
        input.category.trim(),
        input.description.trim(),
        input.active,
    )
    .await
    .map_err(|error| unique_error(error, "slug or name"))?
        == 0
    {
        return Err(ApiError::NotFound);
    }

    repo::replace_behaviors(&mut tx, id, &behaviors)
        .await
        .map_err(behaviors_in_use)?;
    tx.commit().await?;

    Ok(())
}

pub async fn remove(pool: &PgPool, ctx: Ctx, id: i64) -> Result<(), ApiError> {
    ctx.require_admin()?;

    let deleted = repo::delete(pool, id).await.map_err(|error| {
        if let sqlx::Error::Database(_) = error {
            ApiError::BadRequest(
                "ORGANIZATION_HAS_INFRASTRUCTURE: elimina o mueve antes sus redes".into(),
            )
        } else {
            ApiError::from(error)
        }
    })?;

    if deleted == 0 {
        return Err(ApiError::NotFound);
    }

    Ok(())
}

/// Selector de PCs. Un NPC asignado se muestra tambien: el POST de asignacion lo mueve a
/// la nueva red en una sola consulta, asi que el administrador no tiene que desasignarlo.
pub async fn npc_catalog(pool: &PgPool, ctx: Ctx) -> Result<Vec<NpcCatalogResponse>, ApiError> {
    ctx.require_admin()?;
    Ok(repo::find_npc_catalog(pool)
        .await?
        .into_iter()
        .map(NpcCatalogResponse::from)
        .collect())
}

pub async fn create_network(
    pool: &PgPool,
    ctx: Ctx,
    organization: i64,
    input: NetworkInput,
) -> Result<i64, ApiError> {
    ctx.require_admin()?;
    validate_network(&input)?;

    if repo::find_by_id(pool, organization).await?.is_none() {
        return Err(ApiError::NotFound);
    }

    repo::insert_network(pool, organization, input.name.trim(), &input.zone)
        .await
        .map_err(|error| unique_error(error, "network name"))
}

pub async fn edit_network(
    pool: &PgPool,
    ctx: Ctx,
    organization: i64,
    network: i64,
    input: NetworkInput,
) -> Result<(), ApiError> {
    ctx.require_admin()?;
    validate_network(&input)?;

    if repo::update_network(pool, organization, network, input.name.trim(), &input.zone).await? == 0
    {
        return Err(ApiError::NotFound);
    }

    Ok(())
}

pub async fn remove_network(
    pool: &PgPool,
    ctx: Ctx,
    organization: i64,
    network: i64,
) -> Result<(), ApiError> {
    ctx.require_admin()?;

    repo::delete_network(pool, organization, network)
        .await
        .map_err(|error| {
            if let sqlx::Error::Database(_) = error {
                ApiError::BadRequest("NETWORK_IN_USE: mueve antes los NPCs a otra red".into())
            } else {
                ApiError::from(error)
            }
        })
        .and_then(|deleted| {
            if deleted == 0 {
                Err(ApiError::NotFound)
            } else {
                Ok(())
            }
        })
}

pub async fn assign_network_npc(
    pool: &PgPool,
    ctx: Ctx,
    organization: i64,
    network: i64,
    input: NetworkNpcInput,
) -> Result<(), ApiError> {
    ctx.require_admin()?;
    validate_network_npc(&input)?;

    if !repo::network_belongs_to(pool, organization, network).await? {
        return Err(ApiError::NotFound);
    }

    if !repo::npc_exists(pool, input.npc_id).await? {
        return Err(ApiError::BadRequest("unknown npc".into()));
    }

    repo::assign_npc(pool, network, input.npc_id, input.role.trim()).await?;
    Ok(())
}

/// Mueve el nodo por el mapa y le cambia el rol. La zona no se toca: sale del hardware, y
/// para pasar a Internet se le instala una interfaz de red desde el editor de PC.
pub async fn edit_node(pool: &PgPool, ctx: Ctx, organization: i64, network: i64, npc: i64, input: NodeInput) -> Result<(), ApiError> {
    ctx.require_admin()?;
    if !is_code(input.role.trim(), 40) {
        return Err(ApiError::BadRequest("invalid node role".into()));
    }
    if !(-10_000..=10_000).contains(&input.map_x) || !(-10_000..=10_000).contains(&input.map_y) {
        return Err(ApiError::BadRequest("node coordinates are outside the map".into()));
    }
    if repo::update_node(pool, organization, network, npc, input.map_x, input.map_y, input.role.trim()).await? == 0 {
        return Err(ApiError::NotFound);
    }
    Ok(())
}

/// Quita un nodo del mapa: primero sus enlaces LAN, despues su sitio en la red. El NPC y su
/// hardware siguen existiendo, asi que el nodo se puede volver a asignar mas tarde.
pub async fn remove_node(pool: &PgPool, ctx: Ctx, organization: i64, network: i64, npc: i64) -> Result<(), ApiError> {
    ctx.require_admin()?;

    let mut tx = pool.begin().await?;
    repo::delete_node_links(&mut tx, organization, npc).await?;
    let deleted = repo::delete_node(&mut tx, organization, network, npc).await?;

    if deleted == 0 {
        // Sin commit: si el nodo no era de esta organizacion, sus cables tampoco se tocan.
        return Err(ApiError::NotFound);
    }

    tx.commit().await?;
    Ok(())
}

/// Replaces the machine inside an existing node. The admin flow is deliberately creative:
/// it manufactures installed components and never reads or changes a player's inventory.
pub async fn save_computer(pool: &PgPool, ctx: Ctx, organization: i64, network: i64, npc: i64, input: ComputerBuildInput) -> Result<(), ApiError> {
    ctx.require_admin()?;
    if !repo::node_belongs_to(pool, organization, network, npc).await? { return Err(ApiError::NotFound); }
    if input.kit_id.is_some() && !input.items.is_empty() { return Err(ApiError::BadRequest("choose a kit or a custom build, not both".into())); }

    let mut tx = pool.begin().await?;
    let (build, source_kit) = if let Some(kit_id) = input.kit_id {
        let kit = kits::find_kit(&mut tx, kit_id).await?.ok_or_else(|| ApiError::BadRequest("unknown kit".into()))?;
        let items = kits::lock_items(&mut tx, kit.id).await?;
        (items.into_iter().map(|item| (item.part, item.amount)).collect::<Vec<_>>(), Some(kit.id))
    } else {
        if input.items.is_empty() { return Err(ApiError::BadRequest("a custom build needs pieces".into())); }
        let ids: Vec<i64> = input.items.iter().map(|item| item.part).collect();
        if ids.iter().any(|id| *id <= 0) || input.items.iter().any(|item| item.amount <= 0) { return Err(ApiError::BadRequest("invalid part or amount".into())); }
        let mut parts = Vec::with_capacity(ids.len());
        for item in &input.items {
            let part = hardware::find_part(&mut tx, item.part).await?.ok_or_else(|| ApiError::BadRequest("unknown part".into()))?;
            parts.push((part, item.amount));
        }
        (parts, None)
    };
    fitting::check_build(&build)?;
    let machine = match hardware::find_npc_machine(&mut tx, npc).await? {
        Some(machine) => machine,
        None => hardware::insert_npc_machine(&mut tx, npc, "node-pc").await?,
    };
    hardware::clear_machine_components(&mut tx, machine).await?;
    for (part, amount) in &build { for _ in 0..*amount { hardware::insert_npc_installed(&mut tx, part.id, npc, machine).await?; } }
    hardware::recompute_machine(&mut tx, machine).await?;
    repo::set_node_source_kit(&mut tx, network, npc, source_kit).await?;
    tx.commit().await?;
    Ok(())
}

/// Reads the installed parts, not a UI draft, so reopening Computer always starts from
/// the machine that was committed for this node.
pub async fn computer_configuration(pool: &PgPool, ctx: Ctx, organization: i64, network: i64, npc: i64) -> Result<ComputerConfigurationResponse, ApiError> {
    ctx.require_admin()?;
    if !repo::node_belongs_to(pool, organization, network, npc).await? { return Err(ApiError::NotFound); }
    let (source_kit_id, items) = repo::computer_configuration(pool, network, npc).await?;
    Ok(ComputerConfigurationResponse { source_kit_id, items: items.into_iter().map(|(part, amount)| ComputerBuildItemInput { part, amount }).collect() })
}

/// WEB is a virtual edge node. A public node reserves one physical LAN port for it;
/// it is never stored as an NPC or exposed as a hackable machine.


pub async fn create_link(pool: &PgPool, ctx: Ctx, organization: i64, input: NodeLinkInput) -> Result<i64, ApiError> { ctx.require_admin()?; if input.from_npc_id <= 0 || input.to_npc_id <= 0 || input.from_npc_id == input.to_npc_id { return Err(ApiError::BadRequest("select two different nodes".into())); } if repo::available_lan_ports(pool, organization, input.from_npc_id).await? < 1 || repo::available_lan_ports(pool, organization, input.to_npc_id).await? < 1 { return Err(ApiError::BadRequest("LAN_PORT_UNAVAILABLE: one of the nodes has no free LAN port".into())); } Ok(repo::insert_link(pool, organization, input.from_npc_id, input.to_npc_id).await?) }
pub async fn remove_link(pool: &PgPool, ctx: Ctx, organization: i64, link: i64) -> Result<(), ApiError> { ctx.require_admin()?; if repo::delete_link(pool, organization, link).await? == 0 { Err(ApiError::NotFound) } else { Ok(()) } }

/// Crea un PC completo para la red. El kit se materializa en componentes propios del NPC,
/// igual que al comprar un kit, pero sin pago ni limite de maquinas de jugador.
pub async fn create_server(
    pool: &PgPool,
    ctx: Ctx,
    organization: i64,
    network: i64,
    input: CreateServerInput,
) -> Result<ServerCreatedResponse, ApiError> {
    ctx.require_admin()?;
    validate_server(&input)?;

    if !repo::network_belongs_to(pool, organization, network).await? {
        return Err(ApiError::NotFound);
    }

    let mut tx = pool.begin().await?;
    let kit = kits::find_kit(&mut tx, input.kit_id)
        .await?
        .ok_or_else(|| ApiError::BadRequest("unknown kit".into()))?;
    let items = kits::lock_items(&mut tx, kit.id).await?;

    if items.is_empty() {
        return Err(ApiError::BadRequest("este kit no lleva piezas".into()));
    }

    let build: Vec<_> = items
        .iter()
        .map(|item| (item.part.clone(), item.amount))
        .collect();
    fitting::check_build(&build)?;

    let ip: i64 = sqlx::query_scalar("SELECT random_free_ip()")
        .fetch_one(&mut *tx)
        .await?;
    let password: String = rand::rng()
        .sample_iter(Alphanumeric)
        .take(8)
        .map(char::from)
        .collect();
    let npc = repo::insert_npc(&mut tx, input.npc_type, ip, &password).await?;
    repo::insert_npc_info(&mut tx, npc, input.name.trim(), input.web.trim()).await?;
    repo::insert_npc_log(&mut tx, npc).await?;

    // `hardware.name` conserva el limite historico de 15 caracteres; el nombre visible
    // es npc_info.name y no se trunca.
    let hardware_name: String = input.name.trim().chars().take(15).collect();
    let machine = hardware::insert_npc_machine(&mut tx, npc, &hardware_name).await?;
    let mut components = Vec::new();

    for item in &items {
        for _ in 0..item.amount {
            components
                .push(hardware::insert_npc_installed(&mut tx, item.part.id, npc, machine).await?);
        }
    }

    hardware::recompute_machine(&mut tx, machine).await?;
    repo::assign_npc_with_kit(&mut tx, network, npc, input.role.trim(), kit.id).await?;
    tx.commit().await?;

    Ok(ServerCreatedResponse {
        npc_id: npc,
        ip,
        password,
        machine_id: machine,
        kit_id: kit.id,
        components,
    })
}

/// Un banco con cuentas abiertas no puede dejar de serlo: la clave ajena de `bank_accounts`
/// apunta al comportamiento `banking`, y quitarlo dejaria esas cuentas sin banco. La base lo
/// impide; aqui se traduce, porque un 500 no le dice al administrador que cierre las cuentas.
fn behaviors_in_use(error: sqlx::Error) -> ApiError {
    let culpable = match &error {
        sqlx::Error::Database(inner) => inner.constraint(),
        _ => None,
    };

    if culpable == Some("bank_accounts_bank_is_a_bank_fkey") {
        return ApiError::BadRequest(
            "BANK_IN_USE: tiene cuentas abiertas; cierralas antes de dejar de ser banco".into(),
        );
    }

    ApiError::from(error)
}

fn unique_error(error: sqlx::Error, field: &str) -> ApiError {
    if let sqlx::Error::Database(_) = error {
        ApiError::BadRequest(format!("ya existe una organizacion con ese {field}"))
    } else {
        ApiError::from(error)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn organization() -> OrganizationInput {
        OrganizationInput {
            slug: "central-bank".into(),
            name: "Central Bank".into(),
            category: "bank".into(),
            description: String::new(),
            active: true,
            behaviors: vec![BehaviorInput {
                code: "banking".into(),
                config: json!({"currency": "USD"}),
            }],
        }
    }

    #[test]
    fn accepts_a_declarative_behavior() {
        assert!(validate(&organization()).is_ok());
    }

    #[test]
    fn rejects_an_executable_or_ambiguous_behavior_config() {
        let mut input = organization();
        input.behaviors[0].config = json!("run this");
        assert!(validate(&input).is_err());
    }

    #[test]
    fn rejects_repeated_behavior_codes() {
        let mut input = organization();
        input.behaviors.push(BehaviorInput {
            code: "banking".into(),
            config: json!({}),
        });
        assert!(validate(&input).is_err());
    }

    #[test]
    fn accepts_all_network_zones() {
        for zone in ZONES {
            assert!(
                validate_network(&NetworkInput {
                    name: "Main segment".into(),
                    zone: zone.into(),
                })
                .is_ok()
            );
        }
    }

    #[test]
    fn rejects_an_unknown_network_zone() {
        assert!(
            validate_network(&NetworkInput {
                name: "Main segment".into(),
                zone: "internet".into(),
            })
            .is_err()
        );
    }

    #[test]
    fn rejects_a_server_without_a_valid_role() {
        assert!(
            validate_server(&CreateServerInput {
                name: "Payments gateway".into(),
                kit_id: 1,
                role: "web server".into(),
                npc_type: 70,
                web: String::new(),
            })
            .is_err()
        );
    }
}

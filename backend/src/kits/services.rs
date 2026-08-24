use std::collections::{HashMap, HashSet};

use sqlx::PgPool;

use crate::{
    auth::Ctx,
    error::ApiError,
    hardware::{fitting, repo as hardware, services::MAX_MACHINES},
};

use super::{
    dtos::{
        BuyKitRequest,
        KitBoughtResponse,
        KitClanResponse,
        KitInput,
        KitResponse,
    },
    models::{KitClanRow, KitItemRow, KitRow},
    repo,
};

/// Lista cerrada, igual que los tipos de pieza: cada ambito nuevo es una regla de quien lo
/// ve, y eso no se decide desde un formulario.
const SCOPES: [&str; 4] = ["shop", "marketplace", "clan", "hidden"];

//
// ==============================
// LECTURA
// ==============================
//

/// Junta cada kit con sus lineas y sus corporaciones en una sola pasada.
fn assemble(
    kits: Vec<KitRow>,
    items: Vec<KitItemRow>,
    clans: Vec<KitClanRow>,
) -> Vec<KitResponse> {
    let mut by_kit: HashMap<i64, Vec<KitItemRow>> = HashMap::new();

    for item in items {
        by_kit.entry(item.kit_id).or_default().push(item);
    }

    let mut clans_by_kit: HashMap<i64, Vec<KitClanRow>> = HashMap::new();

    for clan in clans {
        clans_by_kit.entry(clan.kit_id).or_default().push(clan);
    }

    kits.into_iter()
        .map(|kit| {
            let items = by_kit.remove(&kit.id).unwrap_or_default();
            let clans = clans_by_kit.remove(&kit.id).unwrap_or_default();

            KitResponse::new(kit, items, clans)
        })
        .collect()
}

async fn fill(pool: &PgPool, kits: Vec<KitRow>) -> Result<Vec<KitResponse>, ApiError> {
    if kits.is_empty() {
        return Ok(Vec::new());
    }

    let ids: Vec<i64> = kits.iter().map(|kit| kit.id).collect();

    let items = repo::find_items(pool, &ids).await?;
    let clans = repo::find_clans(pool, &ids).await?;

    Ok(assemble(kits, items, clans))
}

/// Los kits que este jugador puede comprar.
pub async fn offered(pool: &PgPool, user_id: i64) -> Result<Vec<KitResponse>, ApiError> {
    let kits = repo::find_offered(pool, user_id).await?;

    fill(pool, kits).await
}

pub async fn list(pool: &PgPool, ctx: Ctx) -> Result<Vec<KitResponse>, ApiError> {
    ctx.require_admin()?;

    let kits = repo::find_kits(pool).await?;

    fill(pool, kits).await
}

/// Las corporaciones que existen, para el selector del panel.
pub async fn clans(pool: &PgPool, ctx: Ctx) -> Result<Vec<KitClanResponse>, ApiError> {
    ctx.require_admin()?;

    let clans = repo::all_clans(pool)
        .await?
        .into_iter()
        .map(KitClanResponse::from)
        .collect();

    Ok(clans)
}

//
// ==============================
// VALIDACION
// ==============================
//

/// Lo que se comprueba antes de guardar un kit.
///
/// La parte que decide si el equipo encaja no vive aqui: es `fitting::check_build`, la
/// misma que usa la pantalla de hardware al instalar una pieza. Un kit que pase por aqui se
/// puede montar pieza a pieza, y uno que no, no llega a guardarse.
async fn checked(pool: &PgPool, input: &KitInput) -> Result<Vec<(i64, i16)>, ApiError> {
    if input.name.trim().is_empty() {
        return Err(ApiError::BadRequest("name is required".into()));
    }

    if !SCOPES.contains(&input.scope.as_str()) {
        return Err(ApiError::BadRequest(format!(
            "scope must be one of {}",
            SCOPES.join(", ")
        )));
    }

    if input.price < 0 {
        return Err(ApiError::BadRequest("price cannot be negative".into()));
    }

    if input.items.is_empty() {
        return Err(ApiError::BadRequest("un kit necesita piezas".into()));
    }

    // Un kit de corporacion sin corporacion no lo veria nadie, y enlazar corporaciones a un
    // kit que no es de corporacion guardaria un dato que nunca se lee.
    if input.scope == "clan" && input.clans.is_empty() {
        return Err(ApiError::BadRequest(
            "CLAN_REQUIRED: un kit de corporacion necesita al menos una".into(),
        ));
    }

    if input.scope != "clan" && !input.clans.is_empty() {
        return Err(ApiError::BadRequest(
            "solo un kit de ambito clan se enlaza a corporaciones".into(),
        ));
    }

    let mut seen = HashSet::new();
    let mut wanted = Vec::with_capacity(input.items.len());

    for item in &input.items {
        if item.amount < 1 || item.amount > 32 {
            return Err(ApiError::BadRequest("amount must be between 1 and 32".into()));
        }

        // La clave primaria ya lo impediria, pero el mensaje seria el de la base de datos.
        if !seen.insert(item.part) {
            return Err(ApiError::BadRequest(
                "DUPLICATE_PART: repite una pieza; usa la cantidad".into(),
            ));
        }

        wanted.push((item.part, item.amount));
    }

    let ids: Vec<i64> = wanted.iter().map(|(part, _)| *part).collect();
    let found = repo::find_parts_by_id(pool, &ids).await?;

    if found.len() != ids.len() {
        return Err(ApiError::BadRequest("unknown part".into()));
    }

    let by_id: HashMap<i64, _> = found.into_iter().map(|part| (part.id, part)).collect();

    // En el mismo orden que llegaron, para que el error senale la pieza que el panel
    // muestra en esa posicion.
    let build: Vec<_> = wanted
        .iter()
        .map(|(part, amount)| (by_id[part].clone(), *amount))
        .collect();

    fitting::check_build(&build)?;

    Ok(wanted)
}

//
// ==============================
// PANEL
// ==============================
//

pub async fn create(pool: &PgPool, ctx: Ctx, input: KitInput) -> Result<i64, ApiError> {
    ctx.require_admin()?;

    let items = checked(pool, &input).await?;

    let mut tx = pool.begin().await?;

    let id = repo::insert_kit(
        &mut tx,
        input.name.trim(),
        input.description.trim(),
        input.price,
        &input.scope,
        input.listed,
    )
    .await
    .map_err(|error| name_taken(error, input.name.trim()))?;

    repo::replace_items(&mut tx, id, &items).await?;
    repo::replace_clans(&mut tx, id, &input.clans).await?;

    tx.commit().await?;

    Ok(id)
}

pub async fn edit(pool: &PgPool, ctx: Ctx, id: i64, input: KitInput) -> Result<(), ApiError> {
    ctx.require_admin()?;

    let items = checked(pool, &input).await?;

    let mut tx = pool.begin().await?;

    let changed = repo::update_kit(
        &mut tx,
        id,
        input.name.trim(),
        input.description.trim(),
        input.price,
        &input.scope,
        input.listed,
    )
    .await
    .map_err(|error| name_taken(error, input.name.trim()))?;

    if changed == 0 {
        return Err(ApiError::BadRequest("unknown kit".into()));
    }

    repo::replace_items(&mut tx, id, &items).await?;
    repo::replace_clans(&mut tx, id, &input.clans).await?;

    tx.commit().await?;

    Ok(())
}

fn name_taken(error: sqlx::Error, name: &str) -> ApiError {
    if let sqlx::Error::Database(_) = error {
        ApiError::BadRequest(format!("ya existe un kit llamado {name}"))
    } else {
        ApiError::from(error)
    }
}

/// Borrar un kit no toca ningun equipo ya montado: sus componentes son piezas propias del
/// jugador desde el momento en que se compro.
pub async fn remove(pool: &PgPool, ctx: Ctx, id: i64) -> Result<(), ApiError> {
    ctx.require_admin()?;

    let mut tx = pool.begin().await?;

    let kit = repo::find_kit(&mut tx, id)
        .await?
        .ok_or_else(|| ApiError::BadRequest("unknown kit".into()))?;

    tx.rollback().await?;

    // Sin kit por defecto, un jugador nuevo naceria con la maquina vacia y sin poder
    // cargar nada. Se cambia primero cual es el de arranque y luego se borra este.
    if kit.is_default {
        return Err(ApiError::BadRequest(
            "KIT_IS_DEFAULT: es el equipo de arranque; marca otro antes de borrarlo".into(),
        ));
    }

    if repo::delete_kit(pool, id).await? == 0 {
        return Err(ApiError::BadRequest("unknown kit".into()));
    }

    Ok(())
}

/// Marcar cual es el equipo de arranque. `None` deja de haber uno.
pub async fn set_default(
    pool: &PgPool,
    ctx: Ctx,
    id: i64,
    on: bool,
) -> Result<(), ApiError> {
    ctx.require_admin()?;

    if !on {
        repo::clear_default(pool).await?;
        return Ok(());
    }

    let mut tx = pool.begin().await?;

    if repo::set_default(&mut tx, id).await? == 0 {
        return Err(ApiError::BadRequest("unknown kit".into()));
    }

    tx.commit().await?;

    Ok(())
}

//
// ==============================
// COMPRA
// ==============================
//

/// Comprar un kit monta un equipo entero: una maquina nueva con todas sus piezas puestas.
///
/// Es lo que lo distingue de comprar pieza a pieza, que deja todo en el cajon. Las piezas
/// siguen siendo piezas: se pueden sacar, mover a otra maquina y vender igual que el resto.
pub async fn buy(
    pool: &PgPool,
    user_id: i64,
    kit_id: i64,
    input: BuyKitRequest,
) -> Result<KitBoughtResponse, ApiError> {
    // Que el kit se le ofrezca a este jugador se resuelve con la misma consulta que pinta
    // la tienda, para que no haya dos definiciones de 'visible'.
    let offered = repo::find_offered(pool, user_id).await?;

    let kit = offered
        .into_iter()
        .find(|kit| kit.id == kit_id)
        .ok_or_else(|| ApiError::BadRequest("unknown kit".into()))?;

    let mut tx = pool.begin().await?;

    let items = repo::lock_items(&mut tx, kit.id).await?;

    if items.is_empty() {
        return Err(ApiError::BadRequest("este kit no lleva piezas".into()));
    }

    // El catalogo pudo cambiar despues de guardarse el kit. Se vuelve a comprobar aqui para
    // que nadie acabe con un equipo que la pantalla de hardware considera imposible.
    let build: Vec<_> = items
        .iter()
        .map(|item| (item.part.clone(), item.amount))
        .collect();

    fitting::check_build(&build)?;

    let machines = hardware::count_machines(&mut tx, user_id).await?;

    if machines >= MAX_MACHINES {
        return Err(ApiError::BadRequest(format!(
            "TOO_MANY_MACHINES: ya tienes {machines} equipos, el maximo es {MAX_MACHINES}"
        )));
    }

    let cash = hardware::lock_account_cash(&mut tx, input.account, user_id)
        .await?
        .unwrap_or(0);

    if cash < kit.price {
        return Err(ApiError::BadRequest("insufficient funds".into()));
    }

    if kit.price > 0 {
        hardware::charge_account(&mut tx, input.account, user_id, kit.price).await?;
        hardware::record_spend(&mut tx, user_id, kit.price).await?;
    }

    let name = format!("Server #{}", machines + 1);
    let machine = hardware::insert_machine(&mut tx, user_id, &name).await?;

    let mut made = Vec::new();

    for item in &items {
        for _ in 0..item.amount {
            made.push(hardware::insert_installed(&mut tx, item.part.id, user_id, machine).await?);
        }
    }

    hardware::recompute_machine(&mut tx, machine).await?;

    tx.commit().await?;

    Ok(KitBoughtResponse {
        machine,
        machine_name: name,
        kit: kit.name,
        components: made,
        paid: kit.price,
    })
}

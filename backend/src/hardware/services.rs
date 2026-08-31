use sqlx::PgPool;

use crate::{auth::Ctx, error::ApiError};

use super::{
    dtos::{
        BuyMachineRequest,
        BuyPartRequest,
        ChipsetInput,
        ChipsetResponse,
        ComponentResponse,
        GrantRequest,
        InstallRequest,
        LinkRequest,
        LinkResponse,
        PartInput,
        PartResponse,
        MachineBoughtResponse,
        MachineDetailResponse,
        MachineQuoteResponse,
        MachineResponse,
        MoveMachineRequest,
        RenameMachineRequest,
        PartBoughtResponse,
        SlotsResponse,
    },
    models::ComponentRow,
    fitting,
    repo,
};

const KINDS: [&str; 5] = ["cpu", "ram", "hdd", "net", "mobo"];

/// Lista cerrada: cada tipo nuevo seria una regla de compatibilidad nueva, y eso no se
/// decide desde un formulario. Los chipsets si son abiertos, porque solo son una etiqueta.
const TIERS: [&str; 4] = ["domestica", "gaming", "server", "industrial"];

/// Cuantos equipos puede tener un jugador.
///
/// Sin techo, comprar equipos seria una forma de sumar capacidad sin limite: lo que cuenta
/// el juego es la suma de todas las maquinas de alguien, no la mejor.
pub const MAX_MACHINES: i64 = 4;

/// Lo que cuesta el segundo equipo. Cada uno despues vale el doble que el anterior, asi que
/// el cuarto sale por cuatro veces el segundo. Tener flota es una decision, no un tramite.
const MACHINE_BASE_PRICE: i64 = 25_000;

/// Lo que cuesta el siguiente equipo de alguien que ya tiene `owned`.
fn machine_price(owned: i64) -> i64 {
    MACHINE_BASE_PRICE << (owned - 1).max(0).min(16)
}

//
// ==============================
// MACHINES
// ==============================
//

pub async fn machines(
    pool: &PgPool,
    user_id: i64,
) -> Result<Vec<MachineResponse>, ApiError> {
    let machines =
        repo::find_machines(pool, user_id)
            .await?
            .into_iter()
            .map(MachineResponse::from)
            .collect();

    Ok(machines)
}

/// Cuantos equipos tiene, cuantos caben y lo que cuesta el siguiente.
pub async fn machine_quote(
    pool: &PgPool,
    user_id: i64,
) -> Result<MachineQuoteResponse, ApiError> {
    let mut tx = pool.begin().await?;
    let owned = repo::count_machines(&mut tx, user_id).await?;
    tx.rollback().await?;

    Ok(MachineQuoteResponse {
        machines: owned,
        max: MAX_MACHINES,
        price: machine_price(owned),
        can_buy: owned < MAX_MACHINES,
    })
}

/// El limite lo pone la columna, que admite 15 caracteres.
fn checked_name(name: &str) -> Result<String, ApiError> {
    let name = name.trim();

    if name.is_empty() {
        return Err(ApiError::BadRequest("name is required".into()));
    }

    if name.chars().count() > 15 {
        return Err(ApiError::BadRequest(
            "el nombre no puede pasar de 15 caracteres".into(),
        ));
    }

    Ok(name.to_string())
}

/// Sin nombre, se numera solo. Es como se llama el equipo con el que nace un jugador.
fn machine_name(given: Option<String>, number: i64) -> Result<String, ApiError> {
    match given.as_deref().map(str::trim) {
        None | Some("") => Ok(format!("Server #{number}")),
        Some(name) => checked_name(name),
    }
}

/// Comprar un equipo vacio.
///
/// Nace sin nada dentro: es un sitio donde poner una placa, no un ordenador. Lo que lo
/// convierte en uno son las piezas, que se instalan con las mismas reglas de siempre.
pub async fn buy_machine(
    pool: &PgPool,
    user_id: i64,
    input: BuyMachineRequest,
) -> Result<MachineBoughtResponse, ApiError> {
    let mut tx = pool.begin().await?;

    let owned = repo::count_machines(&mut tx, user_id).await?;

    if owned >= MAX_MACHINES {
        return Err(ApiError::BadRequest(format!(
            "TOO_MANY_MACHINES: ya tienes {owned} equipos, el maximo es {MAX_MACHINES}"
        )));
    }

    let price = machine_price(owned);
    let name = machine_name(input.name, owned + 1)?;

    let cash = repo::lock_account_cash(&mut tx, input.account, user_id)
        .await?
        .unwrap_or(0);

    if cash < price {
        return Err(ApiError::BadRequest("insufficient funds".into()));
    }

    repo::charge_account(&mut tx, input.account, user_id, price).await?;
    repo::record_spend(&mut tx, user_id, price).await?;

    let machine = repo::insert_machine(&mut tx, user_id, &name).await?;

    tx.commit().await?;

    Ok(MachineBoughtResponse {
        machine,
        name,
        paid: price,
    })
}

pub async fn rename_machine(
    pool: &PgPool,
    user_id: i64,
    machine: i64,
    input: RenameMachineRequest,
) -> Result<(), ApiError> {
    let name = checked_name(&input.name)?;

    if repo::rename_machine(pool, machine, user_id, &name).await? == 0 {
        return Err(ApiError::BadRequest("unknown machine".into()));
    }

    Ok(())
}

/// Deshacerse de un equipo.
///
/// Tiene que estar vacio. Vaciarlo pieza a pieza no es burocracia: cada `uninstall` mira si
/// lo que sale deja fuera software ya cargado o guardado, y tirar el equipo entero de golpe
/// se saltaria esa comprobacion. El ultimo equipo tampoco se va: sin ninguno no habria
/// donde poner nada.
pub async fn scrap_machine(
    pool: &PgPool,
    user_id: i64,
    machine: i64,
) -> Result<(), ApiError> {
    let mut tx = pool.begin().await?;

    if !repo::machine_belongs_to(&mut tx, machine, user_id).await? {
        return Err(ApiError::BadRequest("unknown machine".into()));
    }

    let owned = repo::count_machines(&mut tx, user_id).await?;

    if owned <= 1 {
        return Err(ApiError::BadRequest(
            "LAST_MACHINE: es tu unico equipo; no puedes quedarte sin ninguno".into(),
        ));
    }

    let inside = repo::count_in_machine(&mut tx, machine).await?;

    if inside > 0 {
        return Err(ApiError::BadRequest(format!(
            "MACHINE_NOT_EMPTY: quedan {inside} piezas dentro; sacalas antes"
        )));
    }

    repo::delete_machine(&mut tx, machine, user_id).await?;

    tx.commit().await?;

    Ok(())
}

/// Una maquina con lo que tiene puesto y cuanto sitio le queda.
pub async fn machine_detail(
    pool: &PgPool,
    user_id: i64,
) -> Result<Vec<MachineDetailResponse>, ApiError> {
    let machines = repo::find_machines(pool, user_id).await?;
    let owned = repo::find_components(pool, user_id).await?;

    let detail = machines
        .into_iter()
        .map(|machine| {
            let inside: Vec<&ComponentRow> = owned
                .iter()
                .filter(|part| part.installed_in == Some(machine.server_id))
                .collect();

            let board = inside
                .iter()
                .find(|part| part.kind == "mobo")
                .map(|part| ComponentResponse::from((*part).clone()));

            let slots = slots_from(&inside);

            MachineDetailResponse {
                machine: MachineResponse::from(machine),
                board,
                components: inside
                    .into_iter()
                    .filter(|part| part.kind != "mobo")
                    .map(|part| ComponentResponse::from(part.clone()))
                    .collect(),
                slots,
            }
        })
        .collect();

    Ok(detail)
}

/// Ranuras ocupadas y totales. Sin placa todo vale cero: no hay donde enchufar nada.
fn slots_from(inside: &[&ComponentRow]) -> SlotsResponse {
    let board = inside.iter().find(|part| part.kind == "mobo");

    let used = |kind: &str| -> i16 {
        inside.iter().filter(|part| part.kind == kind).count() as i16
    };

    let power = |kind: &str| -> i64 {
        inside
            .iter()
            .filter(|part| part.kind == kind)
            .map(|part| part.power as i64)
            .sum()
    };

    match board {
        None => SlotsResponse::default(),
        Some(board) => SlotsResponse {
            cpu_used: used("cpu"),
            cpu_total: board.cpu_slots,
            ram_used: used("ram"),
            ram_total: board.ram_slots,
            hdd_used: used("hdd"),
            hdd_total: board.hdd_slots,
            net_used: used("net"),
            net_total: board.net_slots,
            cpu_power: power("cpu"),
            cpu_max: board.max_cpu,
            ram_power: power("ram"),
            ram_max: board.max_ram,
            hdd_power: power("hdd"),
            hdd_max: board.max_hdd,
            net_power: power("net"),
            net_max: board.max_net,
            generation: board.generation,
            tier: board.tier.clone(),
        },
    }
}

//
// ==============================
// COMPONENTS
// ==============================
//

pub async fn components(
    pool: &PgPool,
    user_id: i64,
) -> Result<Vec<ComponentResponse>, ApiError> {
    let owned = repo::find_components(pool, user_id)
        .await?
        .into_iter()
        .map(ComponentResponse::from)
        .collect();

    Ok(owned)
}

/// Poner una pieza en una maquina.
///
/// Las reglas de compatibilidad viven en `fitting`, que es el mismo modulo que valida un
/// kit. Aqui solo se cuenta lo que hay puesto ya, que es lo que esta funcion sabe hacer y
/// aquel no: mirar varias filas a la vez.
pub async fn install(
    pool: &PgPool,
    user_id: i64,
    component_id: i64,
    input: InstallRequest,
) -> Result<ComponentResponse, ApiError> {
    let mut tx = pool.begin().await?;

    let part = repo::lock_component(&mut tx, component_id, user_id)
        .await?
        .ok_or_else(|| ApiError::BadRequest("unknown component".into()))?;

    if part.installed_in == Some(input.machine) {
        return Err(ApiError::BadRequest("already installed here".into()));
    }

    if !repo::machine_belongs_to(&mut tx, input.machine, user_id).await? {
        return Err(ApiError::BadRequest("unknown machine".into()));
    }

    let board = repo::find_board(&mut tx, input.machine).await?;

    if part.kind == "mobo" {
        // Una maquina lleva una placa. Cambiarla es sacar la vieja primero.
        if board.is_some() {
            return Err(ApiError::BadRequest("BOARD_ALREADY_PRESENT".into()));
        }
    } else {
        let board = board
            .ok_or_else(|| ApiError::BadRequest("NO_BOARD".into()))?;

        let used_slots = repo::count_installed(&mut tx, input.machine, &part.kind).await?;
        let used_power = repo::installed_power(&mut tx, input.machine, &part.kind).await?;

        fitting::check(
            &fitting::Board::from(&board),
            &fitting::Piece::from(&part),
            used_slots,
            used_power,
        )?;
    }

    // Mover desde otra maquina deja hueco alli, asi que aquella tambien se recalcula.
    let previous = part.installed_in;

    repo::set_installed(&mut tx, component_id, Some(input.machine)).await?;
    repo::recompute_machine(&mut tx, input.machine).await?;

    if let Some(previous) = previous {
        repo::recompute_machine(&mut tx, previous).await?;
    }

    // La tarjeta de red es el interruptor de la direccion publica: ponerla es salir a
    // Internet, y salir a Internet es tener donde te alcancen. Va dentro de la misma
    // transaccion que la instalacion porque son el mismo hecho -- una maquina con tarjeta y
    // sin direccion es un estado que no deberia existir ni un instante.
    if part.kind == "net" {
        repo::assign_public_ip(&mut tx, input.machine).await?;

        // Mover la tarjeta desde otra maquina deja a aquella sin salida, si era la unica.
        if let Some(previous) = previous {
            if repo::count_nics(&mut tx, previous).await? == 0 {
                repo::clear_public_ip(&mut tx, previous).await?;
            }
        }
    }

    tx.commit().await?;

    Ok(ComponentResponse {
        installed_in: Some(input.machine),
        ..ComponentResponse::from(part)
    })
}

/// Sacar una pieza y devolverla al cajon.
///
/// Se bloquea si la capacidad resultante dejaria fuera algo que ya esta guardado o
/// cargado, que es la misma regla que ya impedia comprar una pieza mas pequena.
pub async fn uninstall(
    pool: &PgPool,
    user_id: i64,
    component_id: i64,
) -> Result<ComponentResponse, ApiError> {
    let mut tx = pool.begin().await?;

    let part = repo::lock_component(&mut tx, component_id, user_id)
        .await?
        .ok_or_else(|| ApiError::BadRequest("unknown component".into()))?;

    let machine = part
        .installed_in
        .ok_or_else(|| ApiError::BadRequest("not installed".into()))?;

    if part.kind == "mobo" {
        let inside = repo::count_non_board(&mut tx, machine).await?;

        if inside > 0 {
            return Err(ApiError::BadRequest(format!(
                "BOARD_NOT_EMPTY: quedan {inside} piezas puestas"
            )));
        }
    } else if part.kind == "ram" {
        let left = repo::total_capacity(&mut tx, user_id, "ram").await?
            - part.power as i64;
        let needed = repo::memory_in_use(&mut tx, user_id).await?;

        if left < needed {
            return Err(ApiError::BadRequest(format!(
                "RAM_IN_USE: quedarian {left} MB y hay {needed} MB cargados"
            )));
        }
    } else if part.kind == "hdd" {
        let left = repo::total_capacity(&mut tx, user_id, "hdd").await?
            - part.power as i64;
        let needed = repo::disk_in_use(&mut tx, user_id).await?;

        if left < needed {
            return Err(ApiError::BadRequest(format!(
                "DISK_IN_USE: quedarian {left} MB y hay {needed} MB guardados"
            )));
        }
    }

    repo::set_installed(&mut tx, component_id, None).await?;
    repo::recompute_machine(&mut tx, machine).await?;

    // Sacar la ultima tarjeta saca a la maquina de Internet, y su direccion vuelve al bombo.
    // Se comprueba que no quede ninguna en vez de asumirlo: una placa de servidor admite
    // varias, y quitar una de cuatro no deja a nadie sin salida.
    if part.kind == "net" && repo::count_nics(&mut tx, machine).await? == 0 {
        repo::clear_public_ip(&mut tx, machine).await?;
    }

    tx.commit().await?;

    Ok(ComponentResponse {
        installed_in: None,
        ..ComponentResponse::from(part)
    })
}

//
// ==============================
// SHOP
// ==============================
//

/// El catalogo a la venta. Lo escribe un administrador, no el codigo.
pub async fn shop(pool: &PgPool) -> Result<Vec<PartResponse>, ApiError> {
    let parts = repo::find_parts(pool, true)
        .await?
        .into_iter()
        .map(PartResponse::from)
        .collect();

    Ok(parts)
}

/// Comprar ya no sustituye nada: fabrica la pieza y la deja en el cajon sin instalar.
pub async fn buy(
    pool: &PgPool,
    user_id: i64,
    input: BuyPartRequest,
) -> Result<PartBoughtResponse, ApiError> {
    let mut tx = pool.begin().await?;

    let part = repo::find_part(&mut tx, input.part)
        .await?
        .ok_or_else(|| ApiError::BadRequest("unknown part".into()))?;

    if !part.listed {
        return Err(ApiError::BadRequest("part not for sale".into()));
    }

    let cash = repo::lock_account_cash(&mut tx, input.account, user_id)
        .await?
        .unwrap_or(0);

    if cash < part.price {
        return Err(ApiError::BadRequest("insufficient funds".into()));
    }

    repo::charge_account(&mut tx, input.account, user_id, part.price).await?;
    repo::record_spend(&mut tx, user_id, part.price).await?;

    let component = repo::insert_component(&mut tx, part.id, user_id).await?;

    tx.commit().await?;

    Ok(PartBoughtResponse {
        component,
        kind: part.kind,
        name: part.name,
        power: part.power,
        paid: part.price,
    })
}

//
// ==============================
// ADMIN
// ==============================
//

// El guardia real es el middleware `AdminOnly`, que hace que estas rutas ni existan para
// quien no lo sea. La comprobacion de abajo es la segunda cerradura: si algun dia alguien
// monta estos servicios fuera de ese scope, siguen cerrados. Ya no consulta la base -- el
// middleware dejo el Ctx resuelto -- asi que cuesta nada.

fn validate_part(input: &PartInput) -> Result<(), ApiError> {
    if !KINDS.contains(&input.kind.as_str()) {
        return Err(ApiError::BadRequest(format!(
            "kind must be one of {}",
            KINDS.join(", ")
        )));
    }

    if input.name.trim().is_empty() {
        return Err(ApiError::BadRequest("name is required".into()));
    }

    if !TIERS.contains(&input.tier.as_str()) {
        return Err(ApiError::BadRequest(format!(
            "tier must be one of {}",
            TIERS.join(", ")
        )));
    }

    if input.price < 0 || input.power < 0 {
        return Err(ApiError::BadRequest("price and power cannot be negative".into()));
    }

    // El chipset solo significa algo en la placa que lo tiene y en el procesador que lo
    // exige. En el resto seria un dato que no se lee nunca.
    if input.chipset_id.is_some() && input.kind != "mobo" && input.kind != "cpu" {
        return Err(ApiError::BadRequest(
            "solo una placa o un procesador llevan chipset".into(),
        ));
    }

    // Una placa que no aporta ranuras no sirve para nada, y una pieza que no es placa no
    // tiene ranuras que dar.
    if input.kind == "mobo" {
        let slots = input.cpu_slots + input.ram_slots + input.hdd_slots + input.net_slots;

        if slots <= 0 {
            return Err(ApiError::BadRequest("a board needs at least one slot".into()));
        }

        if input.generation <= 0 {
            return Err(ApiError::BadRequest("a board needs a DDR generation".into()));
        }

        // Un techo a cero rechazaria cualquier pieza de ese tipo, aunque haya ranura.
        let ceilings = [
            ("cpu", input.cpu_slots, input.max_cpu),
            ("ram", input.ram_slots, input.max_ram),
            ("hdd", input.hdd_slots, input.max_hdd),
            ("net", input.net_slots, input.max_net),
        ];

        for (kind, slots, ceiling) in ceilings {
            if slots > 0 && ceiling <= 0 {
                return Err(ApiError::BadRequest(format!(
                    "la placa tiene ranuras de {kind} pero su maximo es 0"
                )));
            }
        }
    } else if input.power <= 0 {
        return Err(ApiError::BadRequest("a part needs power".into()));
    }

    Ok(())
}

pub async fn list_parts(
    pool: &PgPool,
    ctx: Ctx,
) -> Result<Vec<PartResponse>, ApiError> {
    ctx.require_admin()?;

    let parts = repo::find_parts(pool, false)
        .await?
        .into_iter()
        .map(PartResponse::from)
        .collect();

    Ok(parts)
}

pub async fn create_part(
    pool: &PgPool,
    ctx: Ctx,
    input: PartInput,
) -> Result<i64, ApiError> {
    ctx.require_admin()?;
    validate_part(&input)?;

    let id = repo::insert_part(pool, &input).await?;

    Ok(id)
}

pub async fn edit_part(
    pool: &PgPool,
    ctx: Ctx,
    id: i64,
    input: PartInput,
) -> Result<(), ApiError> {
    ctx.require_admin()?;
    validate_part(&input)?;

    let changed = repo::update_part(pool, id, &input).await?;

    if changed == 0 {
        return Err(ApiError::BadRequest("unknown part".into()));
    }

    Ok(())
}

pub async fn remove_part(
    pool: &PgPool,
    ctx: Ctx,
    id: i64,
) -> Result<(), ApiError> {
    ctx.require_admin()?;

    let removed = repo::delete_part(pool, id).await.map_err(|error| {
        // La clave ajena es RESTRICT: si ya hay piezas fabricadas, esto es lo que salta.
        if let sqlx::Error::Database(_) = error {
            ApiError::BadRequest(
                "PART_IN_USE: ya hay componentes de este part; retiralo con listed=false".into(),
            )
        } else {
            ApiError::from(error)
        }
    })?;

    if removed == 0 {
        return Err(ApiError::BadRequest("unknown part".into()));
    }

    Ok(())
}

/// Fabricar componentes de un part y entregarselos a un jugador.
pub async fn grant(
    pool: &PgPool,
    ctx: Ctx,
    input: GrantRequest,
) -> Result<Vec<i64>, ApiError> {
    ctx.require_admin()?;

    if input.amount < 1 || input.amount > 50 {
        return Err(ApiError::BadRequest("amount must be between 1 and 50".into()));
    }

    if !repo::user_exists(pool, input.user).await? {
        return Err(ApiError::BadRequest("unknown user".into()));
    }

    let mut tx = pool.begin().await?;

    if repo::find_part(&mut tx, input.part).await?.is_none() {
        return Err(ApiError::BadRequest("unknown part".into()));
    }

    let mut made = Vec::with_capacity(input.amount as usize);

    for _ in 0..input.amount {
        made.push(repo::insert_component(&mut tx, input.part, input.user).await?);
    }

    tx.commit().await?;

    Ok(made)
}

//
// ==============================
// SERVER STATS
// ==============================
//

/// Sin worker todavia no hay foto, y la pantalla lo dice en vez de romperse.
pub async fn server_stats(
    pool: &PgPool,
) -> Result<serde_json::Value, ApiError> {
    let payload =
        repo::find_server_stats(pool).await?;

    Ok(payload.unwrap_or_else(
        || serde_json::json!({}),
    ))
}

//
// ==============================
// CHIPSETS
// ==============================
//

/// El catalogo de chipsets. Lo ve cualquiera que llegue al panel, porque el formulario de
/// piezas necesita la lista para poder elegir.
pub async fn chipsets(
    pool: &PgPool,
) -> Result<Vec<ChipsetResponse>, ApiError> {
    let chipsets = repo::find_chipsets(pool)
        .await?
        .into_iter()
        .map(ChipsetResponse::from)
        .collect();

    Ok(chipsets)
}

pub async fn create_chipset(
    pool: &PgPool,
    ctx: Ctx,
    input: ChipsetInput,
) -> Result<i64, ApiError> {
    ctx.require_admin()?;

    let name = input.name.trim();

    if name.is_empty() {
        return Err(ApiError::BadRequest("name is required".into()));
    }

    repo::insert_chipset(pool, name).await.map_err(|error| {
        // El nombre es unico: repetirlo es un error del formulario, no del servidor.
        if let sqlx::Error::Database(_) = error {
            ApiError::BadRequest(format!("ya existe un chipset llamado {name}"))
        } else {
            ApiError::from(error)
        }
    })
}

/// Borrar un chipset que alguna pieza usa se rechaza en vez de dejarlo pasar.
///
/// La clave ajena es ON DELETE SET NULL, asi que la base lo permitiria: las piezas se
/// quedarian sin chipset y su compatibilidad cambiaria en silencio. Un procesador que
/// exigia una placa concreta pasaria a valer para todas sin que nadie lo pidiera.
pub async fn remove_chipset(
    pool: &PgPool,
    ctx: Ctx,
    id: i64,
) -> Result<(), ApiError> {
    ctx.require_admin()?;

    let used = repo::chipset_usage(pool, id).await?;

    if used > 0 {
        return Err(ApiError::BadRequest(format!(
            "CHIPSET_IN_USE: {used} piezas lo usan"
        )));
    }

    if repo::delete_chipset(pool, id).await? == 0 {
        return Err(ApiError::BadRequest("unknown chipset".into()));
    }

    Ok(())
}

//
// ==============================
// MAPA DE NODOS
// ==============================
//

/// Colocar un equipo en el mapa.
///
/// El lienzo no tiene bordes duros en el cliente, pero un nodo en -4000 es un nodo que no
/// se puede volver a coger: se recorta a un rango que siempre queda a la vista.
pub async fn move_machine(
    pool: &PgPool,
    user_id: i64,
    machine: i64,
    input: MoveMachineRequest,
) -> Result<(), ApiError> {
    const LIMITE: i32 = 4000;

    let map_x = input.map_x.clamp(0, LIMITE);
    let map_y = input.map_y.clamp(0, LIMITE);

    if repo::move_machine(pool, machine, user_id, map_x, map_y).await? == 0 {
        return Err(ApiError::BadRequest("unknown machine".into()));
    }

    Ok(())
}

pub async fn links(
    pool: &PgPool,
    user_id: i64,
) -> Result<Vec<LinkResponse>, ApiError> {
    let found = repo::find_links(pool, user_id)
        .await?
        .into_iter()
        .map(LinkResponse::from)
        .collect();

    Ok(found)
}

/// Tender un cable entre dos equipos del jugador.
///
/// Cuatro cosas tienen que cumplirse, y ninguna se puede dar por hecha desde el cliente:
/// que los dos sean suyos, que no sean el mismo, que a los dos les quede boca libre, y que
/// no estuvieran ya unidos. La ultima la impone ademas el indice unico de la tabla, que es
/// quien gana si dos peticiones llegan a la vez.
pub async fn create_link(
    pool: &PgPool,
    user_id: i64,
    input: LinkRequest,
) -> Result<LinkResponse, ApiError> {
    let LinkRequest { from, to } = input;

    if from == to {
        return Err(ApiError::BadRequest(
            "SAME_MACHINE: un equipo no se cablea consigo mismo".into(),
        ));
    }

    for machine in [from, to] {
        if !repo::owns_machine(pool, machine, user_id).await? {
            return Err(ApiError::BadRequest("unknown machine".into()));
        }
    }

    for machine in [from, to] {
        let ports = repo::lan_ports(pool, machine).await?;

        if ports == 0 {
            return Err(ApiError::BadRequest(
                "NO_BOARD: un equipo sin placa no tiene bocas de red".into(),
            ));
        }

        // La salida a Internet ocupa boca como cualquier cable. Un equipo con tarjeta de
        // red ya esta enchufado a algo, aunque ese algo no se dibuje como cable: es la
        // misma cuenta que hace el mapa de organizaciones, donde un nodo publico lleva
        // `INTERNET` entre sus conexiones y le come un puerto.
        let uplink = i64::from(repo::has_uplink(pool, machine).await?);
        let usadas = repo::count_links_of(pool, machine).await? + uplink;

        if usadas >= ports {
            return Err(ApiError::BadRequest(format!(
                "NO_LAN_PORTS: esa placa da {ports} bocas y ya hay {usadas} ocupadas \
                 (la salida a Internet cuenta como una)"
            )));
        }
    }

    match repo::insert_link(pool, user_id, from, to).await {
        Ok(row) => Ok(LinkResponse::from(row)),

        // 23505 es la violacion del indice unico del par: ya estaban cableados. Es un
        // error del que pide, no del servidor, y merece decirlo asi.
        Err(sqlx::Error::Database(error)) if error.code().as_deref() == Some("23505") => {
            Err(ApiError::BadRequest(
                "ALREADY_LINKED: esos dos equipos ya estan cableados".into(),
            ))
        }

        Err(error) => Err(error.into()),
    }
}

pub async fn delete_link(
    pool: &PgPool,
    user_id: i64,
    link: i64,
) -> Result<(), ApiError> {
    if repo::delete_link(pool, link, user_id).await? == 0 {
        return Err(ApiError::BadRequest("unknown link".into()));
    }

    Ok(())
}

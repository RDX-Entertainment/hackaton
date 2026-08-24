use serde::{Deserialize, Serialize};

use super::models::{ChipsetRow, ComponentRow, MachineRow, PartRow};

//
// ==============================
// MACHINES
// ==============================
//

#[derive(Debug, Serialize)]
pub struct MachineResponse {
    pub id: i64,
    pub name: String,
    pub cpu: f32,
    pub hdd: f32,
    pub ram: f32,
    pub net: f32,
}

impl From<MachineRow> for MachineResponse {
    fn from(value: MachineRow) -> Self {
        Self {
            id: value.server_id,
            name: value.name,
            cpu: value.cpu,
            hdd: value.hdd,
            ram: value.ram,
            net: value.net,
        }
    }
}

/// Lo que una maquina tiene puesto y cuanto sitio le queda.
#[derive(Debug, Serialize)]
pub struct MachineDetailResponse {
    pub machine: MachineResponse,
    pub board: Option<ComponentResponse>,
    pub components: Vec<ComponentResponse>,
    pub slots: SlotsResponse,
}

#[derive(Debug, Serialize, Default)]
pub struct SlotsResponse {
    pub cpu_used: i16,
    pub cpu_total: i16,
    pub ram_used: i16,
    pub ram_total: i16,
    pub hdd_used: i16,
    pub hdd_total: i16,
    pub net_used: i16,
    pub net_total: i16,

    /// Cuanto suma lo instalado, y hasta donde admite la placa.
    pub cpu_power: i64,
    pub cpu_max: i32,
    pub ram_power: i64,
    pub ram_max: i32,
    pub hdd_power: i64,
    pub hdd_max: i32,
    pub net_power: i64,
    pub net_max: i32,

    /// Generacion DDR que acepta la placa, o 0 si no hay placa.
    pub generation: i16,

    /// Tipo que impone la placa, o vacio si no hay placa.
    pub tier: String,
}

//
// ==============================
// PARTS
// ==============================
//

#[derive(Debug, Serialize)]
pub struct PartResponse {
    pub id: i64,
    pub kind: String,
    pub name: String,
    pub power: i32,
    pub price: i64,
    pub generation: i16,
    pub tier: String,
    pub cpu_slots: i16,
    pub ram_slots: i16,
    pub hdd_slots: i16,
    pub net_slots: i16,
    pub max_cpu: i32,
    pub max_ram: i32,
    pub max_hdd: i32,
    pub max_net: i32,
    pub chipset_id: Option<i64>,
    pub chipset: Option<String>,
    pub listed: bool,
}

impl From<PartRow> for PartResponse {
    fn from(value: PartRow) -> Self {
        Self {
            id: value.id,
            kind: value.kind,
            name: value.name,
            power: value.power,
            price: value.price,
            generation: value.generation,
            tier: value.tier,
            cpu_slots: value.cpu_slots,
            ram_slots: value.ram_slots,
            hdd_slots: value.hdd_slots,
            net_slots: value.net_slots,
            max_cpu: value.max_cpu,
            max_ram: value.max_ram,
            max_hdd: value.max_hdd,
            max_net: value.max_net,
            chipset_id: value.chipset_id,
            chipset: value.chipset,
            listed: value.listed,
        }
    }
}

/// Alta y edicion de una parte desde el panel de administracion.
#[derive(Debug, Deserialize)]
pub struct PartInput {
    pub kind: String,
    pub name: String,
    #[serde(default)]
    pub power: i32,
    pub price: i64,
    #[serde(default)]
    pub generation: i16,
    #[serde(default = "domestic")]
    pub tier: String,
    #[serde(default)]
    pub chipset_id: Option<i64>,
    #[serde(default)]
    pub max_cpu: i32,
    #[serde(default)]
    pub max_ram: i32,
    #[serde(default)]
    pub max_hdd: i32,
    #[serde(default)]
    pub max_net: i32,
    #[serde(default)]
    pub cpu_slots: i16,
    #[serde(default)]
    pub ram_slots: i16,
    #[serde(default)]
    pub hdd_slots: i16,
    #[serde(default)]
    pub net_slots: i16,
    #[serde(default = "listed_by_default")]
    pub listed: bool,
}

fn listed_by_default() -> bool {
    true
}

fn domestic() -> String {
    "domestica".to_owned()
}

//
// ==============================
// CHIPSETS
// ==============================
//

#[derive(Debug, Serialize)]
pub struct ChipsetResponse {
    pub id: i64,
    pub name: String,
}

impl From<ChipsetRow> for ChipsetResponse {
    fn from(value: ChipsetRow) -> Self {
        Self {
            id: value.id,
            name: value.name,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ChipsetInput {
    pub name: String,
}

//
// ==============================
// COMPONENTS
// ==============================
//

#[derive(Debug, Serialize)]
pub struct ComponentResponse {
    pub id: i64,
    pub part_id: i64,
    pub kind: String,
    pub name: String,
    pub power: i32,
    pub generation: i16,
    pub tier: String,

    /// Maquina donde esta puesto, o null si esta en el cajon.
    pub installed_in: Option<i64>,

    // Solo dicen algo en las placas: lo que conceden a la maquina.
    pub cpu_slots: i16,
    pub ram_slots: i16,
    pub hdd_slots: i16,
    pub net_slots: i16,
    pub max_cpu: i32,
    pub max_ram: i32,
    pub max_hdd: i32,
    pub max_net: i32,
    pub chipset_id: Option<i64>,
    pub chipset: Option<String>,
}

impl From<ComponentRow> for ComponentResponse {
    fn from(value: ComponentRow) -> Self {
        Self {
            id: value.id,
            part_id: value.part_id,
            kind: value.kind,
            name: value.name,
            power: value.power,
            generation: value.generation,
            tier: value.tier,
            installed_in: value.installed_in,
            cpu_slots: value.cpu_slots,
            ram_slots: value.ram_slots,
            hdd_slots: value.hdd_slots,
            net_slots: value.net_slots,
            max_cpu: value.max_cpu,
            max_ram: value.max_ram,
            max_hdd: value.max_hdd,
            max_net: value.max_net,
            chipset_id: value.chipset_id,
            chipset: value.chipset,
        }
    }
}

/// Comprar un equipo vacio. El nombre es opcional: sin el, se numera solo.
#[derive(Debug, Deserialize)]
pub struct BuyMachineRequest {
    pub account: i64,

    #[serde(default)]
    pub name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RenameMachineRequest {
    pub name: String,
}

/// Lo que hace falta saber antes de pulsar "comprar equipo": cuantos tienes, cuantos
/// caben y lo que cuesta el siguiente.
#[derive(Debug, Serialize)]
pub struct MachineQuoteResponse {
    pub machines: i64,
    pub max: i64,
    pub price: i64,
    pub can_buy: bool,
}

#[derive(Debug, Serialize)]
pub struct MachineBoughtResponse {
    pub machine: i64,
    pub name: String,
    pub paid: i64,
}

#[derive(Debug, Deserialize)]
pub struct InstallRequest {
    pub machine: i64,
}

/// Fabricar componentes y entregarselos a alguien. Solo administradores.
#[derive(Debug, Deserialize)]
pub struct GrantRequest {
    pub part: i64,
    pub user: i64,
    #[serde(default = "one")]
    pub amount: i32,
}

fn one() -> i32 {
    1
}

//
// ==============================
// SHOP
// ==============================
//

#[derive(Debug, Deserialize)]
pub struct BuyPartRequest {
    pub part: i64,
    pub account: i64,
}

/// Comprar ya no instala nada: entrega la pieza al cajon.
#[derive(Debug, Serialize)]
pub struct PartBoughtResponse {
    pub component: i64,
    pub kind: String,
    pub name: String,
    pub power: i32,
    pub paid: i64,
}

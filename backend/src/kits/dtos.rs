use serde::{Deserialize, Serialize};

use super::models::{KitClanRow, KitItemRow, KitRow};

//
// ==============================
// ENTRADA
// ==============================
//

#[derive(Debug, Deserialize)]
pub struct KitItemInput {
    pub part: i64,

    #[serde(default = "one")]
    pub amount: i16,
}

fn one() -> i16 {
    1
}

#[derive(Debug, Deserialize)]
pub struct KitInput {
    pub name: String,

    #[serde(default)]
    pub description: String,

    #[serde(default)]
    pub price: i64,

    pub scope: String,

    #[serde(default)]
    pub listed: bool,

    pub items: Vec<KitItemInput>,

    /// Solo se leen cuando el ambito es `clan`. Vacio en el resto.
    #[serde(default)]
    pub clans: Vec<i64>,
}

#[derive(Debug, Deserialize)]
pub struct BuyKitRequest {
    pub account: i64,
}

//
// ==============================
// SALIDA
// ==============================
//

#[derive(Debug, Serialize)]
pub struct KitItemResponse {
    pub part: i64,
    pub kind: String,
    pub name: String,
    pub power: i32,
    pub price: i64,
    pub tier: String,
    pub generation: i16,
    pub chipset: Option<String>,
    pub amount: i16,
}

impl From<KitItemRow> for KitItemResponse {
    fn from(value: KitItemRow) -> Self {
        Self {
            part: value.part.id,
            kind: value.part.kind,
            name: value.part.name,
            power: value.part.power,
            price: value.part.price,
            tier: value.part.tier,
            generation: value.part.generation,
            chipset: value.part.chipset,
            amount: value.amount,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct KitClanResponse {
    pub id: i64,
    pub name: String,
    pub nick: String,
}

impl From<KitClanRow> for KitClanResponse {
    fn from(value: KitClanRow) -> Self {
        Self {
            id: value.clan_id,
            name: value.clan_name,
            nick: value.clan_nick,
        }
    }
}

/// Lo que suma el equipo que monta el kit. Es la misma cuenta que hara la maquina cuando
/// se compre, adelantada para que se vea antes de pagar.
#[derive(Debug, Serialize, Default)]
pub struct KitTotals {
    pub cpu: i64,
    pub ram: i64,
    pub hdd: i64,
    pub net: i64,

    /// Cuanto costarian las piezas sueltas. Al lado del precio del kit ensena el descuento.
    pub parts_price: i64,

    pub pieces: i16,
}

#[derive(Debug, Serialize)]
pub struct KitResponse {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub price: i64,
    pub scope: String,
    pub is_default: bool,
    pub listed: bool,
    pub items: Vec<KitItemResponse>,
    pub clans: Vec<KitClanResponse>,
    pub totals: KitTotals,
}

impl KitResponse {
    pub fn new(kit: KitRow, items: Vec<KitItemRow>, clans: Vec<KitClanRow>) -> Self {
        let mut totals = KitTotals::default();

        for item in &items {
            let power = item.part.power as i64 * item.amount as i64;

            match item.part.kind.as_str() {
                "cpu" => totals.cpu += power,
                "ram" => totals.ram += power,
                "hdd" => totals.hdd += power,
                "net" => totals.net += power,
                _ => {}
            }

            totals.parts_price += item.part.price * item.amount as i64;
            totals.pieces += item.amount;
        }

        Self {
            id: kit.id,
            name: kit.name,
            description: kit.description,
            price: kit.price,
            scope: kit.scope,
            is_default: kit.is_default,
            listed: kit.listed,
            items: items.into_iter().map(KitItemResponse::from).collect(),
            clans: clans.into_iter().map(KitClanResponse::from).collect(),
            totals,
        }
    }
}

/// Lo que devuelve comprar un kit: la maquina que se acaba de montar.
#[derive(Debug, Serialize)]
pub struct KitBoughtResponse {
    pub machine: i64,
    pub machine_name: String,
    pub kit: String,
    pub components: Vec<i64>,
    pub paid: i64,
}

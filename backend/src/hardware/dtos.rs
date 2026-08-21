use serde::{Deserialize, Serialize};

use super::models::MachineRow;

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

//
// ==============================
// SHOP
// ==============================
//

#[derive(Debug, Serialize)]
pub struct ShopPartResponse {
    pub kind: &'static str,
    pub id: u8,
    pub name: &'static str,
    pub power: i32,
    pub price: i64,
}

#[derive(Debug, Deserialize)]
pub struct BuyPartRequest {
    pub kind: String,
    pub part: u8,
    pub account: i64,
}

#[derive(Debug, Serialize)]
pub struct PartBoughtResponse {
    pub kind: &'static str,
    pub name: &'static str,
    pub power: i32,
    pub paid: i64,
}

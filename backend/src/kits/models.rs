use sqlx::FromRow;

use crate::hardware::models::PartRow;

/// Un kit: la descripcion de un equipo entero, no un equipo.
#[derive(Debug, Clone, FromRow)]
pub struct KitRow {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub price: i64,

    /// Donde se ofrece: shop, marketplace, clan o hidden.
    pub scope: String,

    /// El equipo con el que nace un jugador nuevo. Como mucho uno en toda la tabla.
    pub is_default: bool,

    pub listed: bool,
}

/// Una linea del kit. Lleva la pieza entera porque quien lo lee siempre la necesita: para
/// pintarla, para sumar su precio y para validar que el conjunto encaja.
#[derive(Debug, Clone, FromRow)]
pub struct KitItemRow {
    pub kit_id: i64,
    pub amount: i16,

    #[sqlx(flatten)]
    pub part: PartRow,
}

/// Una corporacion enlazada a un kit.
#[derive(Debug, Clone, FromRow)]
pub struct KitClanRow {
    pub kit_id: i64,
    pub clan_id: i64,
    pub clan_name: String,
    pub clan_nick: String,
}

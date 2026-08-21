use serde::Serialize;

/// Una entrada de catálogo heredado: el nombre legible y el id numérico con el que viaja.
/// La comparten los tipos de software y las acciones de proceso, que en el legado eran dos
/// switch distintos sobre la misma forma.
#[derive(Serialize)]
pub struct CatalogEntry {
    pub name: &'static str,
    pub id: u8,
}

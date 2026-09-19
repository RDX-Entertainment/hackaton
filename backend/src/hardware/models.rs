use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct MachineRow {
    pub is_gateway: bool,
    pub node_kind: String,
    pub switch_ports: i16,
    pub operational: bool,
    pub disabled_reasons: Vec<String>,
    pub server_id: i64,
    pub name: String,
    pub cpu: f32,
    pub hdd: f32,
    pub ram: f32,
    pub net: f32,
    /// Donde lo puso el jugador en el mapa de nodos.
    pub map_x: i32,
    pub map_y: i32,
    /// Como se la alcanza desde fuera. `None` = sin tarjeta de red, no sale a Internet.
    pub public_ip: Option<i64>,
    /// Como se la nombra dentro de la red de su dueno. La tienen todas.
    pub lan_ip: Option<i64>,
}

/// Un cable LAN entre dos equipos del jugador.
#[derive(Debug, Clone, FromRow)]
pub struct LinkRow {
    pub id: i64,
    pub from_server: i64,
    pub to_server: i64,
}

/// Plantilla de pieza. Define QUE es un componente, no de quien es.
#[derive(Debug, Clone, FromRow)]
pub struct PartRow {
    pub id: i64,
    pub kind: String,
    pub name: String,
    pub power: i32,
    pub price: i64,

    /// Generacion DDR. Solo significa algo en 'ram' y en 'mobo'.
    pub generation: i16,

    /// Ranuras: cuantas piezas caben. Solo significan algo en 'mobo'.
    pub cpu_slots: i16,
    pub ram_slots: i16,
    pub hdd_slots: i16,
    pub net_slots: i16,

    /// Techos: cuanto pueden sumar. Solo significan algo en 'mobo'.
    pub max_cpu: i32,
    pub max_ram: i32,
    pub max_hdd: i32,
    pub max_net: i32,

    /// En una placa es el tipo que impone; en el resto, el tipo que es.
    pub tier: String,

    pub chipset_id: Option<i64>,
    pub chipset: Option<String>,

    pub listed: bool,
}

/// Instancia concreta, propiedad de alguien. `installed_in` en NULL es el cajon.
#[derive(Debug, Clone, FromRow)]
pub struct ComponentRow {
    pub id: i64,
    pub part_id: i64,
    pub installed_in: Option<i64>,

    // Copiados del part al consultar, para que ninguna pantalla tenga que cruzarlo aparte.
    pub kind: String,
    pub name: String,
    pub power: i32,
    pub generation: i16,
    pub cpu_slots: i16,
    pub ram_slots: i16,
    pub hdd_slots: i16,
    pub net_slots: i16,
    pub max_cpu: i32,
    pub max_ram: i32,
    pub max_hdd: i32,
    pub max_net: i32,
    pub tier: String,
    pub chipset_id: Option<i64>,
    pub chipset: Option<String>,
}

/// Lo que una placa impone a la maquina donde esta puesta.
#[derive(Debug, Clone, FromRow)]
pub struct BoardRow {
    pub generation: i16,
    pub tier: String,
    pub chipset_id: Option<i64>,
    pub cpu_slots: i16,
    pub ram_slots: i16,
    pub hdd_slots: i16,
    pub net_slots: i16,
    pub max_cpu: i32,
    pub max_ram: i32,
    pub max_hdd: i32,
    pub max_net: i32,
}

/// Un chipset del catalogo. Los crea un administrador desde el panel.
#[derive(Debug, Clone, FromRow)]
pub struct ChipsetRow {
    pub id: i64,
    pub name: String,
}

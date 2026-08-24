//! Las reglas de compatibilidad, en un solo sitio y sin tocar la base de datos.
//!
//! Se usan desde dos lados que no se parecen: instalar una pieza mira lo que ya hay puesto
//! en una maquina, y validar un kit recorre una lista que todavia no existe en ninguna
//! parte. Si cada uno llevara su copia, en dos semanas dirian cosas distintas y un kit
//! podria describir un equipo que la propia pantalla de hardware rechaza montar.
//!
//! Por eso aqui no hay consultas: entran los numeros ya contados y sale el veredicto.

use crate::error::ApiError;

use super::models::{BoardRow, ComponentRow, PartRow};

/// Lo que una placa impone, venga de una fila instalada o del catalogo.
#[derive(Debug, Clone)]
pub struct Board {
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

/// Una pieza cualquiera, mirada solo por lo que decide si encaja.
#[derive(Debug, Clone)]
pub struct Piece {
    pub kind: String,
    pub name: String,
    pub power: i32,
    pub generation: i16,
    pub tier: String,
    pub chipset_id: Option<i64>,
}

impl Board {
    /// Cuantas piezas de un tipo caben.
    pub fn slots(&self, kind: &str) -> i16 {
        match kind {
            "cpu" => self.cpu_slots,
            "ram" => self.ram_slots,
            "hdd" => self.hdd_slots,
            "net" => self.net_slots,
            _ => 0,
        }
    }

    /// Cuanto pueden sumar entre todas. Distinto de las ranuras: ocho zocalos de memoria y
    /// un techo de 32 GB no dicen lo mismo, y una placa real tiene los dos limites.
    pub fn ceiling(&self, kind: &str) -> i32 {
        match kind {
            "cpu" => self.max_cpu,
            "ram" => self.max_ram,
            "hdd" => self.max_hdd,
            "net" => self.max_net,
            _ => 0,
        }
    }
}

pub fn unit_for(kind: &str) -> &'static str {
    match kind {
        "cpu" => "MHz",
        "ram" | "hdd" => "MB",
        "net" => "Mbit",
        _ => "",
    }
}

/// Si esta pieza entra en esta placa, contando lo que ya hay dentro.
///
/// `used_slots` y `used_power` son lo ocupado ANTES de meter esta pieza.
pub fn check(
    board: &Board,
    piece: &Piece,
    used_slots: i64,
    used_power: i64,
) -> Result<(), ApiError> {
    // El tipo lo impone la placa: una pieza de servidor no entra en una domestica.
    if piece.tier != board.tier {
        return Err(ApiError::BadRequest(format!(
            "TIER_MISMATCH: la placa es {}, {} es {}",
            board.tier, piece.name, piece.tier
        )));
    }

    // El chipset ata el procesador a la placa, igual que el socket en una real. Una pieza
    // sin chipset declarado no ata nada: vale para cualquier placa. Es lo que permite que
    // el catalogo antiguo -- creado sin este campo -- siga sirviendo.
    if piece.kind == "cpu" {
        if let Some(required) = piece.chipset_id {
            if board.chipset_id != Some(required) {
                return Err(ApiError::BadRequest(format!(
                    "CHIPSET_MISMATCH: {} no encaja en el chipset de esta placa",
                    piece.name
                )));
            }
        }
    }

    // La generacion solo limita la memoria: es la unica pieza con DDR.
    if piece.kind == "ram" && piece.generation != board.generation {
        return Err(ApiError::BadRequest(format!(
            "DDR_MISMATCH: la placa acepta DDR{}, {} es DDR{}",
            board.generation, piece.name, piece.generation
        )));
    }

    let slots = board.slots(&piece.kind);

    if used_slots >= slots as i64 {
        return Err(ApiError::BadRequest(format!(
            "NO_SLOTS: la placa tiene {slots} ranuras de {}",
            piece.kind
        )));
    }

    let ceiling = board.ceiling(&piece.kind);
    let total = used_power + piece.power as i64;

    if total > ceiling as i64 {
        return Err(ApiError::BadRequest(format!(
            "OVER_CAPACITY: la placa admite {ceiling} {unit} de {kind} y quedarian {total}",
            unit = unit_for(&piece.kind),
            kind = piece.kind,
        )));
    }

    Ok(())
}

/// Un equipo entero descrito como lista de piezas, sin que exista todavia.
///
/// Es lo que valida un kit antes de dejar que se guarde: si el resultado no se podria
/// montar pieza a pieza, no tiene sentido poder venderlo montado.
pub fn check_build(parts: &[(PartRow, i16)]) -> Result<(), ApiError> {
    let boards: Vec<&(PartRow, i16)> =
        parts.iter().filter(|(part, _)| part.kind == "mobo").collect();

    let board = match boards.as_slice() {
        [] => {
            return Err(ApiError::BadRequest(
                "NO_BOARD: un kit necesita una placa".into(),
            ));
        }
        [single] => *single,
        _ => {
            return Err(ApiError::BadRequest(
                "TWO_BOARDS: un equipo lleva una placa, no varias".into(),
            ));
        }
    };

    if board.1 != 1 {
        return Err(ApiError::BadRequest(format!(
            "TWO_BOARDS: el kit lleva {} placas y un equipo solo admite una",
            board.1
        )));
    }

    let board = Board::from(&board.0);

    // Se acumula igual que lo haria la maquina real, en el mismo orden en que se
    // instalarian, para que el mensaje de error sea el que veria el jugador.
    let mut used_slots = [0i64; 4];
    let mut used_power = [0i64; 4];

    let index = |kind: &str| match kind {
        "cpu" => Some(0usize),
        "ram" => Some(1usize),
        "hdd" => Some(2usize),
        "net" => Some(3usize),
        _ => None,
    };

    for (part, amount) in parts.iter().filter(|(part, _)| part.kind != "mobo") {
        let Some(slot) = index(&part.kind) else {
            continue;
        };

        let piece = Piece::from(part);

        for _ in 0..*amount {
            check(&board, &piece, used_slots[slot], used_power[slot])?;

            used_slots[slot] += 1;
            used_power[slot] += part.power as i64;
        }
    }

    Ok(())
}

impl From<&BoardRow> for Board {
    fn from(row: &BoardRow) -> Self {
        Self {
            generation: row.generation,
            tier: row.tier.clone(),
            chipset_id: row.chipset_id,
            cpu_slots: row.cpu_slots,
            ram_slots: row.ram_slots,
            hdd_slots: row.hdd_slots,
            net_slots: row.net_slots,
            max_cpu: row.max_cpu,
            max_ram: row.max_ram,
            max_hdd: row.max_hdd,
            max_net: row.max_net,
        }
    }
}

impl From<&PartRow> for Board {
    fn from(row: &PartRow) -> Self {
        Self {
            generation: row.generation,
            tier: row.tier.clone(),
            chipset_id: row.chipset_id,
            cpu_slots: row.cpu_slots,
            ram_slots: row.ram_slots,
            hdd_slots: row.hdd_slots,
            net_slots: row.net_slots,
            max_cpu: row.max_cpu,
            max_ram: row.max_ram,
            max_hdd: row.max_hdd,
            max_net: row.max_net,
        }
    }
}

impl From<&ComponentRow> for Piece {
    fn from(row: &ComponentRow) -> Self {
        Self {
            kind: row.kind.clone(),
            name: row.name.clone(),
            power: row.power,
            generation: row.generation,
            tier: row.tier.clone(),
            chipset_id: row.chipset_id,
        }
    }
}

impl From<&PartRow> for Piece {
    fn from(row: &PartRow) -> Self {
        Self {
            kind: row.kind.clone(),
            name: row.name.clone(),
            power: row.power,
            generation: row.generation,
            tier: row.tier.clone(),
            chipset_id: row.chipset_id,
        }
    }
}

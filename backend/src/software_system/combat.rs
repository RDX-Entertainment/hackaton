//! Reglas puras de combate del software. No consulta la base ni genera aleatoriedad global:
//! el caller inyecta los lanzamientos y persiste el resultado antes de que el worker actúe.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombatAttribute {
    Effectiveness,
    Penetration,
    Stealth,
    Manipulation,
    AntiForensics,
    Persistence,
}

impl CombatAttribute {
    pub const ALL: [Self; 6] = [
        Self::Effectiveness,
        Self::Penetration,
        Self::Stealth,
        Self::Manipulation,
        Self::AntiForensics,
        Self::Persistence,
    ];

    pub const fn code(self) -> &'static str {
        match self {
            Self::Effectiveness => "effectiveness",
            Self::Penetration => "penetration",
            Self::Stealth => "stealth",
            Self::Manipulation => "manipulation",
            Self::AntiForensics => "anti_forensics",
            Self::Persistence => "persistence",
        }
    }

    pub const fn counter(self) -> &'static str {
        match self {
            Self::Effectiveness => "resistance",
            Self::Penetration => "hardening",
            Self::Stealth => "detection",
            Self::Manipulation => "integrity",
            Self::AntiForensics => "forensics",
            Self::Persistence => "removal",
        }
    }
}

pub const PROBABILITY_MIN: i32 = 2;
pub const PROBABILITY_MAX: i32 = 98;
/// Variación por cada punto de diferencia. Se mantiene baja para que una ventaja pequeña
/// no convierta la operación en una certeza.
pub const PROBABILITY_FACTOR: f32 = 0.8;

/// La curva centralizada: con diferencia cero da 50%, y cada punto de ventaja desplaza 0.8%.
pub fn probability(attacker: i32, defender: i32) -> i32 {
    ((50.0 + (attacker - defender) as f32 * PROBABILITY_FACTOR).round() as i32)
        .clamp(PROBABILITY_MIN, PROBABILITY_MAX)
}

pub trait RollSource {
    /// Devuelve un entero inclusivo entre 1 y 100.
    fn roll_percent(&mut self) -> u8;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Resolution {
    pub probability: i32,
    pub roll: u8,
    pub success: bool,
}

pub fn resolve(attacker: i32, defender: i32, rolls: &mut impl RollSource) -> Resolution {
    let probability = probability(attacker, defender);
    let roll = rolls.roll_percent().clamp(1, 100);
    Resolution {
        probability,
        roll,
        success: i32::from(roll) <= probability,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Fixed(u8);
    impl RollSource for Fixed {
        fn roll_percent(&mut self) -> u8 {
            self.0
        }
    }

    #[test]
    fn every_offensive_attribute_has_one_unique_counter() {
        let mut seen = std::collections::BTreeSet::new();
        for attribute in CombatAttribute::ALL {
            assert!(seen.insert(attribute.counter()));
        }
        assert_eq!(seen.len(), 6);
    }

    #[test]
    fn equal_attributes_are_even() {
        assert_eq!(probability(42, 42), 50);
    }

    #[test]
    fn balance_curve_uses_the_configured_factor() {
        assert_eq!(probability(10, 0), 58);
        assert_eq!(probability(25, 0), 70);
        assert_eq!(probability(0, 25), 30);
        assert_eq!(probability(50, 0), 90);
    }

    #[test]
    fn probability_is_bounded() {
        assert_eq!(probability(0, 100), PROBABILITY_MIN);
        assert_eq!(probability(100, 0), PROBABILITY_MAX);
    }

    #[test]
    fn injected_roll_makes_outcome_deterministic() {
        let mut success = Fixed(50);
        let mut failure = Fixed(51);
        assert!(resolve(10, 10, &mut success).success);
        assert!(!resolve(10, 10, &mut failure).success);
    }
}

//! Registro declarativo de habilidades: umbral + capacidad compatible. Las pantallas sólo
//! muestran este resultado; la autorización permanece en los servicios.
use super::dtos::Stats;

pub struct SkillRule {
    pub capability: &'static str,
    pub attribute: &'static str,
    pub action: &'static str,
    pub threshold: i32,
}
pub const RULES: &[SkillRule] = &[
    SkillRule {
        capability: "web_manipulator",
        attribute: "manipulation",
        action: "modify_web",
        threshold: 25,
    },
    SkillRule {
        capability: "forensic_scrubber",
        attribute: "anti_forensics",
        action: "hide_evidence",
        threshold: 25,
    },
    SkillRule {
        capability: "forensic_scrubber",
        attribute: "anti_forensics",
        action: "forge_evidence",
        threshold: 50,
    },
    SkillRule {
        capability: "forensic_analyzer",
        attribute: "forensics",
        action: "analyze_evidence",
        threshold: 25,
    },
    SkillRule {
        capability: "virus_infect",
        attribute: "persistence",
        action: "install_persistence",
        threshold: 50,
    },
    SkillRule {
        capability: "antivirus",
        attribute: "removal",
        action: "remove_persistence",
        threshold: 50,
    },
    SkillRule {
        capability: "ssh_exploit",
        attribute: "penetration",
        action: "escalate_privileges",
        threshold: 50,
    },
];
pub fn value(stats: &Stats, attribute: &str) -> i32 {
    match attribute {
        "manipulation" => stats.manipulation,
        "anti_forensics" => stats.anti_forensics,
        "forensics" => stats.forensics,
        "persistence" => stats.persistence,
        "removal" => stats.removal,
        "penetration" => stats.penetration,
        _ => 0,
    }
}
pub fn unlocked(
    capability: Option<&str>,
    stats: &Stats,
) -> Vec<(&'static str, &'static str, i32, bool)> {
    RULES
        .iter()
        .filter(|rule| Some(rule.capability) == capability)
        .map(|rule| {
            (
                rule.action,
                rule.attribute,
                rule.threshold,
                value(stats, rule.attribute) >= rule.threshold,
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn web_edit_requires_compatible_capability_and_threshold() {
        let mut stats = Stats::default();
        stats.manipulation = 24;
        assert!(!unlocked(Some("web_manipulator"), &stats)[0].3);
        stats.manipulation = 25;
        assert!(unlocked(Some("web_manipulator"), &stats)[0].3);
        assert!(!unlocked(Some("ssh_exploit"), &stats)
            .iter()
            .any(|rule| rule.0 == "modify_web"));
    }

    #[test]
    fn forge_has_a_higher_threshold_than_hiding() {
        let mut stats = Stats::default();
        stats.anti_forensics = 30;
        let rules = unlocked(Some("forensic_scrubber"), &stats);
        assert!(rules.iter().any(|rule| rule.0 == "hide_evidence" && rule.3));
        assert!(
            rules
                .iter()
                .any(|rule| rule.0 == "forge_evidence" && !rule.3)
        );
    }
}

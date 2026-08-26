use sqlx::{PgPool, Row};
use crate::error::ApiError;
use super::dtos::Stats;

/// Los atributos van de 0 a 100, pero el consumo no: se compara contra `hardware.cpu/ram/net`,
/// que estan en las unidades del catalogo de piezas. Una maquina de arranque trae 1000 de CPU,
/// 512 de RAM y **1 de red** —la NET1, la pieza mas baja de la escala—, asi que un consumo
/// inventado en la escala de los atributos hacia imposible lanzar nada: pedia 5 de red a
/// quien tiene 1. Un programa pelado cabe en la maquina de arranque y los modulos son los que
/// obligan a subir de pieza.
pub(crate) fn base_stats(role: &str) -> Stats {
    if role == "defender" { Stats { effectiveness: 45, detection: 50, removal: 35, stealth: 0, penetration: 0, stability: 65, cpu: 200, ram: 128, net: 1, weight: 24 } }
    else { Stats { effectiveness: 50, stealth: 45, detection: 0, removal: 0, penetration: 45, stability: 55, cpu: 200, ram: 128, net: 1, weight: 24 } }
}

pub(crate) fn apply_module(mut stats: Stats, row: &sqlx::postgres::PgRow) -> Stats {
    stats.effectiveness = (stats.effectiveness + row.get::<i32, _>("effectiveness_delta")).clamp(0, 100);
    stats.stealth = (stats.stealth + row.get::<i32, _>("stealth_delta")).clamp(0, 100);
    stats.detection = (stats.detection + row.get::<i32, _>("detection_delta")).clamp(0, 100);
    stats.removal = (stats.removal + row.get::<i32, _>("removal_delta")).clamp(0, 100);
    stats.penetration = (stats.penetration + row.get::<i32, _>("penetration_delta")).clamp(0, 100);
    stats.stability = (stats.stability + row.get::<i32, _>("stability_delta")).clamp(0, 100);
    stats.cpu = (stats.cpu + row.get::<i32, _>("cpu_delta")).max(1);
    stats.ram = (stats.ram + row.get::<i32, _>("ram_delta")).max(1);
    stats.net = (stats.net + row.get::<i32, _>("net_delta")).max(1);
    stats.weight = (stats.weight + row.get::<i32, _>("size_delta")).max(1);
    stats
}

pub(crate) async fn blueprint_modules(pool: &PgPool, blueprint_id: i64) -> Result<Vec<sqlx::postgres::PgRow>, ApiError> {
    Ok(sqlx::query("SELECT m.* FROM blueprint_modules bm JOIN research_modules m ON m.id=bm.module_id WHERE bm.blueprint_id=$1 ORDER BY m.id").bind(blueprint_id).fetch_all(pool).await?)
}

pub(crate) async fn owned_blueprint(pool: &PgPool, user_id: i64, blueprint_id: i64) -> Result<(String, String, String, i32), ApiError> {
    sqlx::query("SELECT b.name,c.code,c.role,b.revision FROM software_blueprints b JOIN software_capabilities c ON c.id=b.capability_id WHERE b.id=$1 AND b.user_id=$2")
        .bind(blueprint_id).bind(user_id).fetch_optional(pool).await?
        .map(|row| (row.get("name"), row.get("code"), row.get("role"), row.get("revision"))).ok_or(ApiError::NotFound)
}

pub(crate) fn legacy_type(capability: &str) -> i16 { match capability { "ssh_exploit" => 14, "ftp_exploit" => 13, "cracker" => 1, "virus_infect" => 99, _ => 4 } }

pub(crate) fn profile_stats(row: &sqlx::postgres::PgRow) -> Stats { Stats { effectiveness: row.get("effectiveness"), stealth: row.get("stealth"), detection: row.get("detection"), removal: row.get("removal"), penetration: row.get("penetration"), stability: row.get("stability"), cpu: row.get("cpu_need"), ram: row.get("ram_need"), net: row.get("net_need"), weight: row.get("weight") } }

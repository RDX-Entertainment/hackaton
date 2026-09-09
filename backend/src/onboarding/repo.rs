use sqlx::PgPool;

use super::models::{
    OnboardingFacts,
    PracticeTargetRow,
};

/// Las seis preguntas, en una sola ida y vuelta.
///
/// Son `EXISTS` y no cuentas porque lo unico que decide un paso es si hay al menos uno, y
/// `EXISTS` para en la primera fila que encuentra.
///
/// La intrusion mira las **dos** listas. `machine_accesses` es la que autoriza y la que
/// enseña la pantalla de Accesos, pero `internet_access` sigue siendo por donde entra quien
/// crackea a la vieja usanza; exigir solo la primera diria «entra en una maquina» a alguien
/// que ya esta dentro de una.
pub async fn facts(
    pool: &PgPool,
    user_id: i64,
) -> Result<OnboardingFacts, sqlx::Error> {
    sqlx::query_as::<_, OnboardingFacts>(
        "SELECT
            EXISTS(SELECT 1 FROM user_research_modules WHERE user_id=$1) AS modules,
            EXISTS(SELECT 1 FROM software_blueprints WHERE user_id=$1) AS blueprints,
            EXISTS(SELECT 1 FROM software
                   WHERE user_id=$1 AND NOT is_npc AND NOT is_folder) AS programs,
            EXISTS(SELECT 1 FROM software_running
                   WHERE user_id=$1 AND NOT is_npc) AS running,
            (EXISTS(SELECT 1 FROM machine_accesses WHERE attacker_id=$1)
             OR EXISTS(SELECT 1 FROM internet_access WHERE user_id=$1)) AS intrusions,
            EXISTS(SELECT 1 FROM processes
                   WHERE creator_id=$1 AND action=91 AND completed_at IS NULL) AS compiling",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await
}

/// El campo de tiro del jugador, por orden de creacion.
pub async fn practice_targets(
    pool: &PgPool,
    user_id: i64,
) -> Result<Vec<PracticeTargetRow>, sqlx::Error> {
    sqlx::query_as::<_, PracticeTargetRow>(
        "SELECT n.npc_ip AS ip, COALESCE(i.name, '') AS name
         FROM npc_purpose p
         JOIN npc n ON n.id = p.npc_id
         LEFT JOIN npc_info i ON i.npc_id = n.id
         WHERE p.kind = 'practice' AND p.owner_id = $1
         ORDER BY n.id",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

use sqlx::PgPool;

use super::models::{
    OnboardingFacts,
    PracticeTargetRow,
    TutorialStepRow,
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

//
// ==============================
// LA DEFINICION
// ==============================
//

/// Los pasos que gobiernan, en orden.
///
/// Solo los activos: un paso apagado sigue escrito --y se sigue editando en el panel-- pero
/// no cierra el camino de nadie. Es la forma de preparar un paso nuevo sin soltarlo encima de
/// quien esta jugando.
pub async fn active_steps(
    pool: &PgPool,
) -> Result<Vec<TutorialStepRow>, sqlx::Error> {
    sqlx::query_as::<_, TutorialStepRow>(
        "SELECT id, position, code, fact, views, actions, focus_view, focus_anchor, hint, active
         FROM tutorial_steps
         WHERE active
         ORDER BY position",
    )
    .fetch_all(pool)
    .await
}

/// Todos, encendidos o no. Es lo que ve el panel.
pub async fn all_steps(
    pool: &PgPool,
) -> Result<Vec<TutorialStepRow>, sqlx::Error> {
    sqlx::query_as::<_, TutorialStepRow>(
        "SELECT id, position, code, fact, views, actions, focus_view, focus_anchor, hint, active
         FROM tutorial_steps
         ORDER BY position",
    )
    .fetch_all(pool)
    .await
}

/// Si el tutorial manda. Apagado sigue diciendo el paso y deja de bloquear.
pub async fn enforced(
    pool: &PgPool,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar::<_, bool>("SELECT enforced FROM tutorial_config WHERE id")
        .fetch_optional(pool)
        .await
        .map(|value| value.unwrap_or(false))
}

pub async fn set_enforced(
    pool: &PgPool,
    value: bool,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO tutorial_config (id, enforced) VALUES (true, $1)
         ON CONFLICT (id) DO UPDATE SET enforced = excluded.enforced",
    )
    .bind(value)
    .execute(pool)
    .await?;

    Ok(())
}

/// Guarda un paso, nuevo o existente, y devuelve su id.
///
/// El hueco de `position` se hace **antes** de escribir, en la misma transaccion: la posicion
/// es unica en la tabla, asi que meter un paso en medio sin apartar a los de detras seria un
/// error de clave duplicada en vez de un paso insertado.
pub async fn save_step(
    pool: &PgPool,
    id: Option<i64>,
    step: &TutorialStepRow,
) -> Result<i64, sqlx::Error> {
    let mut tx = pool.begin().await?;

    sqlx::query(
        "UPDATE tutorial_steps SET position = position + 1
         WHERE position >= $1 AND ($2::bigint IS NULL OR id <> $2)",
    )
    .bind(step.position)
    .bind(id)
    .execute(&mut *tx)
    .await?;

    // Las dos ramas repiten la lista de campos en vez de compartirla. Es a proposito: para
    // compartirla habria que numerar los parametros igual en un `INSERT` de nueve y en un
    // `UPDATE` de diez, y ese apano --colar un parametro que la consulta no usa-- es un error
    // de «bind message supplies 10 parameters» que solo aparece al ejecutar.
    let saved = match id {
        Some(existing) => {
            sqlx::query_scalar::<_, i64>(
                "UPDATE tutorial_steps
                    SET position = $1, code = $2, fact = $3, views = $4, actions = $5,
                        focus_view = $6, focus_anchor = $7, hint = $8, active = $9
                  WHERE id = $10
              RETURNING id",
            )
            .bind(step.position)
            .bind(&step.code)
            .bind(&step.fact)
            .bind(&step.views)
            .bind(&step.actions)
            .bind(&step.focus_view)
            .bind(&step.focus_anchor)
            .bind(&step.hint)
            .bind(step.active)
            .bind(existing)
            .fetch_optional(&mut *tx)
            .await?
        }

        None => {
            sqlx::query_scalar::<_, i64>(
                "INSERT INTO tutorial_steps
                     (position, code, fact, views, actions, focus_view, focus_anchor, hint, active)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                 RETURNING id",
            )
            .bind(step.position)
            .bind(&step.code)
            .bind(&step.fact)
            .bind(&step.views)
            .bind(&step.actions)
            .bind(&step.focus_view)
            .bind(&step.focus_anchor)
            .bind(&step.hint)
            .bind(step.active)
            .fetch_optional(&mut *tx)
            .await?
        }
    };

    let saved = saved.ok_or(sqlx::Error::RowNotFound)?;

    renumber(&mut tx).await?;
    tx.commit().await?;

    Ok(saved)
}

pub async fn delete_step(
    pool: &PgPool,
    id: i64,
) -> Result<bool, sqlx::Error> {
    let mut tx = pool.begin().await?;

    let gone = sqlx::query("DELETE FROM tutorial_steps WHERE id = $1")
        .bind(id)
        .execute(&mut *tx)
        .await?
        .rows_affected()
        > 0;

    renumber(&mut tx).await?;
    tx.commit().await?;

    Ok(gone)
}

/// Deja las posiciones en 1, 2, 3… sin huecos.
///
/// Los huecos no rompen nada --el orden es el que es-- pero se ven en el panel, y un paso
/// numerado 7 en una lista de cinco parece un paso perdido.
async fn renumber(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE tutorial_steps s
            SET position = orden.fila
           FROM (SELECT id, row_number() OVER (ORDER BY position, id) AS fila
                   FROM tutorial_steps) orden
          WHERE s.id = orden.id AND s.position <> orden.fila",
    )
    .execute(&mut **tx)
    .await?;

    Ok(())
}

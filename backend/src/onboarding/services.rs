use sqlx::PgPool;

use crate::error::ApiError;

use super::{
    dtos::{
        OnboardingResponse,
        OnboardingRulesResponse,
        OnboardingStepResponse,
        TutorialConfigInput,
        TutorialResponse,
        TutorialStepInput,
        TutorialVocabularyResponse,
        VocabularyEntry,
    },
    models::{
        OnboardingFacts,
        TutorialStepRow,
    },
    repo,
    vocabulario,
};

/// El paso actual: **el primero cuyo hecho todavia no se cumple**.
///
/// Se mira en orden y se para en el primero, no se busca el ultimo hecho: quien compilo un
/// programa y luego lo borro tiene que volver a compilar, y un maximo diria que ya paso de
/// ahi.
fn current<'a>(
    steps: &'a [TutorialStepRow],
    facts: &OnboardingFacts,
) -> Option<&'a TutorialStepRow> {
    steps
        .iter()
        .find(|step| !facts.get(&step.fact))
}

pub async fn onboarding(
    pool: &PgPool,
    user_id: i64,
) -> Result<OnboardingResponse, ApiError> {
    let definition = repo::active_steps(pool).await?;
    let facts = repo::facts(pool, user_id).await?;
    let enforced = repo::enforced(pool).await?;

    let actual = current(&definition, &facts);

    // Con el tutorial apagado se sigue diciendo el paso y el foco, y las listas viajan
    // vacias de contenido --`None`, que es «todo»--. Asi un cliente que se olvide de mirar
    // `enforced` tampoco apaga nada: la unica forma de que bloquee es que el servidor le
    // mande una lista, y apagado no se la manda.
    let rules = actual.map(|step| OnboardingRulesResponse {
        views: enforced
            .then(|| step.views.clone())
            .flatten(),
        actions: enforced
            .then(|| step.actions.clone())
            .flatten(),
        focus_view: step.focus_view.clone(),
        focus_anchor: step.focus_anchor.clone(),
        hint: step.hint.clone(),
    });

    let steps = definition
        .iter()
        .map(|step| OnboardingStepResponse {
            code: step.code.clone(),
            done: facts.get(&step.fact),
        })
        .collect();

    let practice = repo::practice_targets(pool, user_id)
        .await?
        .into_iter()
        .map(Into::into)
        .collect();

    Ok(OnboardingResponse {
        current: actual.map(|step| step.code.clone()),
        steps,
        compiling: facts.compiling,
        practice,
        rules,
        enforced,
    })
}

//
// ==============================
// EL VETO
// ==============================
//

/// Deja pasar, o corta con `TUTORIAL_LOCKED`.
///
/// Se llama desde el sitio que hace la accion, no desde un middleware. Un middleware tendria
/// que adivinar que accion es cada peticion mirando su ruta y su cuerpo --y «encolar una
/// tarea» son quince acciones distintas en el mismo `POST`--, mientras que aqui lo dice quien
/// ya lo sabe.
///
/// Cuatro razones para dejar pasar, y todas son la misma: **nadie esta en el tutorial**. El
/// tutorial apagado, sin pasos escritos, el jugador que ya los cumplio todos, y el paso que
/// no restringe acciones.
pub async fn guard(
    pool: &PgPool,
    user_id: i64,
    action: &str,
) -> Result<(), ApiError> {
    if !repo::enforced(pool).await? {
        return Ok(());
    }

    let definition = repo::active_steps(pool).await?;

    if definition.is_empty() {
        return Ok(());
    }

    let facts = repo::facts(pool, user_id).await?;

    let Some(step) = current(&definition, &facts) else {
        return Ok(());
    };

    let Some(allowed) = &step.actions else {
        return Ok(());
    };

    if allowed
        .iter()
        .any(|permitida| permitida == action)
    {
        return Ok(());
    }

    // El paso viaja en el mensaje para que el cliente pueda decir **que** hay que hacer en
    // vez de solo que esto no. El codigo, que es lo que se traduce, es lo que va delante de
    // los dos puntos.
    Err(ApiError::BadRequest(format!(
        "TUTORIAL_LOCKED: {}",
        step.code
    )))
}

//
// ==============================
// EL PANEL
// ==============================
//

pub async fn tutorial(
    pool: &PgPool,
) -> Result<TutorialResponse, ApiError> {
    Ok(TutorialResponse {
        enforced: repo::enforced(pool).await?,
        steps: repo::all_steps(pool)
            .await?
            .into_iter()
            .map(Into::into)
            .collect(),
    })
}

pub fn vocabulary() -> TutorialVocabularyResponse {
    TutorialVocabularyResponse {
        facts: vocabulario::FACTS
            .iter()
            .map(|(code, label)| VocabularyEntry { code, label })
            .collect(),

        actions: vocabulario::ACTIONS
            .iter()
            .map(|(code, label)| VocabularyEntry { code, label })
            .collect(),
    }
}

/// Guarda un paso, y antes comprueba que lo que dice se puede comprobar.
///
/// Lo que se valida no es el formato: es que **cada codigo lo sepa cumplir alguien**. Una
/// accion con una errata no es un bloqueo raro, es un bloqueo que no se levanta jamas —el
/// jugador no puede hacer algo que no existe— y el sitio donde eso se ve es este, no la
/// pantalla de un jugador dentro de una semana.
pub async fn save_step(
    pool: &PgPool,
    id: Option<i64>,
    input: TutorialStepInput,
) -> Result<i64, ApiError> {
    let code = input.code.trim().to_owned();

    if code.is_empty() {
        return Err(ApiError::BadRequest(
            "STEP_CODE_REQUIRED".into(),
        ));
    }

    if !vocabulario::is_fact(&input.fact) {
        return Err(ApiError::BadRequest(format!(
            "UNKNOWN_FACT: {}",
            input.fact
        )));
    }

    if let Some(actions) = &input.actions {
        if let Some(rara) = actions
            .iter()
            .find(|action| !vocabulario::is_action(action))
        {
            return Err(ApiError::BadRequest(format!(
                "UNKNOWN_ACTION: {rara}"
            )));
        }
    }

    let step = TutorialStepRow {
        id: id.unwrap_or_default(),
        position: input.position.max(1),
        code,
        fact: input.fact,
        views: input.views,
        actions: input.actions,
        focus_view: vacio_es_nada(input.focus_view),
        focus_anchor: vacio_es_nada(input.focus_anchor),
        hint: vacio_es_nada(input.hint),
        active: input.active,
    };

    repo::save_step(pool, id, &step)
        .await
        .map_err(Into::into)
}

/// Un campo de texto que llega vacio es un campo sin rellenar, no una cadena vacia guardada.
fn vacio_es_nada(value: Option<String>) -> Option<String> {
    value
        .map(|text| text.trim().to_owned())
        .filter(|text| !text.is_empty())
}

pub async fn delete_step(
    pool: &PgPool,
    id: i64,
) -> Result<(), ApiError> {
    if repo::delete_step(pool, id).await? {
        return Ok(());
    }

    Err(ApiError::NotFound)
}

pub async fn set_config(
    pool: &PgPool,
    input: TutorialConfigInput,
) -> Result<(), ApiError> {
    repo::set_enforced(pool, input.enforced)
        .await
        .map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn facts(
        modules: bool,
        blueprints: bool,
        programs: bool,
        running: bool,
        intrusions: bool,
    ) -> OnboardingFacts {
        OnboardingFacts {
            modules,
            blueprints,
            programs,
            running,
            intrusions,
            compiling: false,
        }
    }

    fn step(
        position: i32,
        code: &str,
        fact: &str,
    ) -> TutorialStepRow {
        TutorialStepRow {
            id: position as i64,
            position,
            code: code.to_owned(),
            fact: fact.to_owned(),
            views: None,
            actions: None,
            focus_view: None,
            focus_anchor: None,
            hint: None,
            active: true,
        }
    }

    /// El circuito sembrado por la 0058, que es el de A3.
    fn circuito() -> Vec<TutorialStepRow> {
        vec![
            step(1, "modules", "modules"),
            step(2, "blueprint", "blueprints"),
            step(3, "compile", "programs"),
            step(4, "install", "running"),
            step(5, "intrusion", "intrusions"),
        ]
    }

    /// Una cuenta recien creada nace con los modulos repartidos, asi que su primer paso no
    /// es conseguirlos: es disenar algo con ellos.
    #[test]
    fn a_new_account_starts_at_the_blueprint() {
        let pasos = circuito();
        let actual = current(
            &pasos,
            &facts(true, false, false, false, false),
        );

        assert_eq!(
            actual.map(|paso| paso.code.as_str()),
            Some("blueprint")
        );
    }

    #[test]
    fn a_compiled_program_still_has_to_be_loaded() {
        let pasos = circuito();
        let actual = current(
            &pasos,
            &facts(true, true, true, false, false),
        );

        assert_eq!(
            actual.map(|paso| paso.code.as_str()),
            Some("install")
        );
    }

    /// El que ya entro en una maquina no tiene siguiente paso: juega.
    #[test]
    fn a_player_who_broke_in_is_done() {
        let pasos = circuito();
        let actual = current(
            &pasos,
            &facts(true, true, true, true, true),
        );

        assert!(actual.is_none());
    }

    /// Quien borro lo que habia compilado vuelve a ese paso, aunque mas adelante quede algo
    /// hecho. Por eso se para en el primero que falta y no se busca el maximo.
    #[test]
    fn a_hole_in_the_middle_wins_over_later_progress() {
        let pasos = circuito();
        let actual = current(
            &pasos,
            &facts(true, true, false, false, true),
        );

        assert_eq!(
            actual.map(|paso| paso.code.as_str()),
            Some("compile")
        );
    }

    /// El orden lo pone la tabla, no el codigo: dos pasos intercambiados cambian el paso
    /// actual sin tocar Rust, que es justo el motivo de que la definicion sea un dato.
    #[test]
    fn the_order_comes_from_the_table() {
        let pasos = vec![
            step(1, "intrusion", "intrusions"),
            step(2, "blueprint", "blueprints"),
        ];

        let actual = current(
            &pasos,
            &facts(true, false, false, false, false),
        );

        assert_eq!(
            actual.map(|paso| paso.code.as_str()),
            Some("intrusion")
        );
    }

    /// Un hecho que este servidor no sabe comprobar no atrapa a nadie: cuenta como hecho y
    /// el circuito sigue. Al reves, un paso mal escrito seria una carcel sin llave.
    #[test]
    fn an_unknown_fact_does_not_trap_the_player() {
        let pasos = vec![
            step(1, "raro", "lo_que_sea"),
            step(2, "blueprint", "blueprints"),
        ];

        let actual = current(
            &pasos,
            &facts(true, false, false, false, false),
        );

        assert_eq!(
            actual.map(|paso| paso.code.as_str()),
            Some("blueprint")
        );
    }
}

use sqlx::PgPool;

use crate::error::ApiError;

use super::{
    dtos::{
        OnboardingResponse,
        OnboardingStepResponse,
    },
    models::OnboardingFacts,
    repo,
};

/// El circuito, en el orden en que se recorre.
///
/// Cada paso habilita al siguiente: sin modulos no hay diseño, sin diseño no hay nada que
/// compilar, lo que no esta en memoria no dispara, y hasta que no se dispara contra algo no
/// se ha entrado en ninguna maquina. Es el mismo camino que ya estaba repartido entre cinco
/// pantallas; aqui es una lista.
const STEPS: [&str; 5] = [
    "modules",
    "blueprint",
    "compile",
    "install",
    "intrusion",
];

/// Si cada paso esta hecho, en el orden de `STEPS`.
fn progress(facts: &OnboardingFacts) -> [bool; 5] {
    [
        facts.modules,
        facts.blueprints,
        facts.programs,
        facts.running,
        facts.intrusions,
    ]
}

/// El primer paso sin hacer.
///
/// **Se mira en orden y se para en el primero**, no se busca el ultimo hecho: quien compilo
/// un programa y luego lo borro tiene que volver a compilar, y un maximo diria que ya paso
/// de ahi.
fn current(done: [bool; 5]) -> Option<&'static str> {
    STEPS
        .iter()
        .zip(done)
        .find(|(_, hecho)| !*hecho)
        .map(|(code, _)| *code)
}

pub async fn onboarding(
    pool: &PgPool,
    user_id: i64,
) -> Result<OnboardingResponse, ApiError> {
    let facts = repo::facts(pool, user_id).await?;
    let done = progress(&facts);

    let steps = STEPS
        .iter()
        .zip(done)
        .map(|(code, done)| OnboardingStepResponse {
            code: *code,
            done,
        })
        .collect();

    let practice = repo::practice_targets(pool, user_id)
        .await?
        .into_iter()
        .map(Into::into)
        .collect();

    Ok(OnboardingResponse {
        current: current(done),
        steps,
        compiling: facts.compiling,
        practice,
    })
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

    /// Una cuenta recien creada nace con los modulos repartidos, asi que su primer paso no
    /// es conseguirlos: es diseñar algo con ellos.
    #[test]
    fn a_new_account_starts_at_the_blueprint() {
        let done = progress(&facts(true, false, false, false, false));

        assert_eq!(current(done), Some("blueprint"));
    }

    #[test]
    fn a_compiled_program_still_has_to_be_loaded() {
        let done = progress(&facts(true, true, true, false, false));

        assert_eq!(current(done), Some("install"));
    }

    /// El que ya entro en una maquina no tiene siguiente paso: juega.
    #[test]
    fn a_player_who_broke_in_is_done() {
        let done = progress(&facts(true, true, true, true, true));

        assert_eq!(current(done), None);
    }

    /// Quien borro lo que habia compilado vuelve a ese paso, aunque mas adelante quede algo
    /// hecho. Por eso se para en el primero que falta y no se busca el maximo.
    #[test]
    fn a_hole_in_the_middle_wins_over_later_progress() {
        let done = progress(&facts(true, true, false, false, true));

        assert_eq!(current(done), Some("compile"));
    }
}

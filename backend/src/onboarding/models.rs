use sqlx::FromRow;

/// Los hechos del mundo de los que se deduce el paso.
///
/// Cada campo es una pregunta cerrada al estado real —¿tiene un diseño?, ¿hay algo en
/// memoria?— y no una marca guardada. **No existe una columna `tutorial_step`** a proposito:
/// seria una segunda verdad que se desincroniza en cuanto alguien haga las cosas en otro
/// orden, y lo que se calcula de los hechos no puede mentir. Llegar por otro camino tambien
/// cuenta.
#[derive(Debug, FromRow)]
pub struct OnboardingFacts {
    pub modules: bool,
    pub blueprints: bool,
    pub programs: bool,
    pub running: bool,
    pub intrusions: bool,
    pub compiling: bool,
}

/// Una maquina del campo de tiro: suya, y solo la ve el.
#[derive(Debug, FromRow)]
pub struct PracticeTargetRow {
    pub ip: i64,
    pub name: String,
}

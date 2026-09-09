use serde::Serialize;

use super::models::PracticeTargetRow;

/// Un paso del circuito, en codigo.
///
/// Va el codigo y no la frase por la misma razon que los errores de A1: **el servidor no
/// sabe en que lengua lee quien pregunta**. La frase, el verbo del boton y la pantalla a la
/// que lleva son cosa del cliente, que es quien conoce sus rutas y su idioma.
#[derive(Debug, Serialize)]
pub struct OnboardingStepResponse {
    pub code: &'static str,
    pub done: bool,
}

#[derive(Debug, Serialize)]
pub struct PracticeTargetResponse {
    pub ip: i64,
    pub name: String,
}

impl From<PracticeTargetRow> for PracticeTargetResponse {
    fn from(value: PracticeTargetRow) -> Self {
        Self {
            ip: value.ip,
            name: value.name,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct OnboardingResponse {
    /// El primer paso sin hacer. `null` cuando ya estan todos: entonces el jugador juega y
    /// la pantalla no tiene nada que empujar.
    pub current: Option<&'static str>,

    pub steps: Vec<OnboardingStepResponse>,

    /// Hay una compilacion en marcha. El paso sigue siendo `compile` —todavia no hay
    /// fichero—, pero no hay nada que hacer salvo esperar, y decir «compila» a quien esta
    /// compilando es lo que hace que una pantalla parezca tonta.
    pub compiling: bool,

    /// Las direcciones del campo de tiro. El alta las crea y hasta hoy no las decia nadie:
    /// eran dos maquinas suyas, invisibles, en un internet de cincuenta y dos.
    pub practice: Vec<PracticeTargetResponse>,
}

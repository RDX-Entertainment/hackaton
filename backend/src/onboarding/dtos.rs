use serde::{
    Deserialize,
    Serialize,
};

use super::models::{
    PracticeTargetRow,
    TutorialStepRow,
};

/// Un paso del circuito, en codigo.
///
/// Va el codigo y no la frase por la misma razon que los errores de A1: **el servidor no
/// sabe en que lengua lee quien pregunta**. La frase, el verbo del boton y la pantalla a la
/// que lleva son cosa del cliente, que es quien conoce sus rutas y su idioma.
#[derive(Debug, Serialize)]
pub struct OnboardingStepResponse {
    pub code: String,
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
    /// Permanent graduation: changing software or the tutorial definition cannot undo it.
    pub completed: bool,
    /// El primer paso sin hacer. `null` cuando ya estan todos: entonces el jugador juega y
    /// la pantalla no tiene nada que empujar.
    pub current: Option<String>,

    pub steps: Vec<OnboardingStepResponse>,

    /// Hay una compilacion en marcha. El paso sigue siendo `compile` —todavia no hay
    /// fichero—, pero no hay nada que hacer salvo esperar, y decir «compila» a quien esta
    /// compilando es lo que hace que una pantalla parezca tonta.
    pub compiling: bool,

    /// Las direcciones del campo de tiro. El alta las crea y hasta hoy no las decia nadie:
    /// eran dos maquinas suyas, invisibles, en un internet de cincuenta y dos.
    pub practice: Vec<PracticeTargetResponse>,

    /// Lo que deja hacer el paso actual. `null` cuando no hay paso --el jugador esta suelto--
    /// o cuando el tutorial esta apagado.
    pub rules: Option<OnboardingRulesResponse>,

    /// Si el tutorial manda. Apagado sigue diciendo el paso y deja de bloquear.
    pub enforced: bool,
}

/// Lo que el paso actual deja hacer, y donde mira.
///
/// Viaja al cliente aunque el servidor ya lo comprueba por su cuenta, y no es duplicar la
/// regla: son dos trabajos distintos. El servidor **veta** --sin eso el bloqueo se salta con
/// `curl`-- y el cliente **explica**, apagando lo que no toca con su motivo. Una pantalla que
/// deja pulsar un boton para contestar que no se puede ensena a base de choques.
#[derive(Debug, Serialize)]
pub struct OnboardingRulesResponse {
    /// `null` es «todas las pantallas». La lista vacia es «ninguna».
    pub views: Option<Vec<String>>,
    pub actions: Option<Vec<String>>,
    pub focus_view: Option<String>,
    pub focus_anchor: Option<String>,

    /// La frase escrita a mano, para un paso que el cliente no sepa nombrar.
    pub hint: Option<String>,
}

//
// ==============================
// EL PANEL
// ==============================
//

/// Un paso, como se edita.
#[derive(Debug, Serialize)]
pub struct TutorialStepResponse {
    pub id: i64,
    pub position: i32,
    pub code: String,
    pub fact: String,
    pub views: Option<Vec<String>>,
    pub actions: Option<Vec<String>>,
    pub focus_view: Option<String>,
    pub focus_anchor: Option<String>,
    pub hint: Option<String>,
    pub active: bool,
}

impl From<TutorialStepRow> for TutorialStepResponse {
    fn from(value: TutorialStepRow) -> Self {
        Self {
            id: value.id,
            position: value.position,
            code: value.code,
            fact: value.fact,
            views: value.views,
            actions: value.actions,
            focus_view: value.focus_view,
            focus_anchor: value.focus_anchor,
            hint: value.hint,
            active: value.active,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct TutorialResponse {
    pub enforced: bool,
    pub steps: Vec<TutorialStepResponse>,
}

/// Lo que se guarda de un paso.
///
/// `views` y `actions` son `Option` de verdad: la ausencia significa «todas» y hay que poder
/// distinguirla de la lista vacia, que significa «ninguna». Por eso no llevan `default`.
#[derive(Debug, Deserialize)]
pub struct TutorialStepInput {
    pub position: i32,
    pub code: String,
    pub fact: String,
    pub views: Option<Vec<String>>,
    pub actions: Option<Vec<String>>,
    pub focus_view: Option<String>,
    pub focus_anchor: Option<String>,
    pub hint: Option<String>,
    #[serde(default = "si")]
    pub active: bool,
}

fn si() -> bool {
    true
}

#[derive(Debug, Deserialize)]
pub struct TutorialConfigInput {
    pub enforced: bool,
}

/// Una entrada del vocabulario: el codigo que se guarda y la frase que lee quien edita.
#[derive(Debug, Serialize)]
pub struct VocabularyEntry {
    pub code: &'static str,
    pub label: &'static str,
}

/// Lo que el panel puede ofrecer, dicho por quien lo va a comprobar.
///
/// Se sirve desde el servidor a proposito, aunque el panel podria llevar la lista escrita: lo
/// que se elige aqui lo tiene que saber vetar la API, y dos listas separadas se desincronizan
/// el dia que alguien anada una accion en un lado y no en el otro.
#[derive(Debug, Serialize)]
pub struct TutorialVocabularyResponse {
    pub facts: Vec<VocabularyEntry>,
    pub actions: Vec<VocabularyEntry>,
}

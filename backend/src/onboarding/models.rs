use sqlx::FromRow;

/// Los hechos del mundo de los que se deduce el paso.
///
/// Mientras se aprende se consulta el estado real. Al terminar se guarda una graduación
/// permanente; quitar programas o cambiar la definición nunca reactiva el tutorial.
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

/// Un paso del tutorial, como esta escrito en la tabla.
///
/// La definicion es un dato; el progreso no. Aqui no hay nada que diga por donde va nadie:
/// `fact` dice que pregunta hay que hacerle al mundo para saber si este paso esta cerrado, y
/// la respuesta se calcula cada vez.
#[derive(Debug, FromRow)]
pub struct TutorialStepRow {
    pub id: i64,
    pub position: i32,
    pub code: String,
    pub fact: String,

    /// `None` es «todas». La lista vacia es «ninguna», que tambien se puede escribir.
    pub views: Option<Vec<String>>,
    pub actions: Option<Vec<String>>,

    pub focus_view: Option<String>,
    pub focus_anchor: Option<String>,
    pub hint: Option<String>,
    pub active: bool,
}

impl OnboardingFacts {
    /// La respuesta a uno de los hechos del vocabulario.
    ///
    /// Un hecho que este servidor no sabe comprobar cuenta como **hecho**, no como pendiente.
    /// Es la direccion segura: un paso con un hecho desconocido se salta, mientras que al
    /// reves atraparia al jugador en un paso que no puede cerrar de ninguna manera.
    pub fn get(
        &self,
        fact: &str,
    ) -> bool {
        match fact {
            "modules" => self.modules,
            "blueprints" => self.blueprints,
            "programs" => self.programs,
            "running" => self.running,
            "intrusions" => self.intrusions,
            _ => true,
        }
    }
}

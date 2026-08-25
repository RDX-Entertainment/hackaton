//! El catalogo de ordenes: **es aqui donde se anaden acciones nuevas**.
//!
//! Todo lo que el lenguaje sabe hacer esta descrito en [`CATALOG`] como datos, no como
//! ramas de un `match` repartidas por el modulo. Una orden nueva —un enrutador, un cobro,
//! lo que haga falta— es una entrada mas en esa lista: el parser, el validador y los
//! mensajes de ayuda salen solos de la descripcion.
//!
//! Lo unico que hay que decidir al anadir una es de que tipo es su destino y que
//! parametros admite. Lo demas ya esta escrito.

/// De que tiene que tener pinta el destino de una orden.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    /// Una direccion del juego, `45.123.10.5`.
    Ip,
    /// Un nombre corto: letras, numeros, guion y guion bajo.
    Ident,
    /// Un numero de cuenta del banco.
    Account,
}

/// Un parametro con nombre, de los que van detras del segundo `|`.
#[derive(Debug, Clone, Copy)]
pub struct Param {
    pub name: &'static str,
    pub required: bool,
    pub numeric: bool,
}

/// Que hace el navegador con la orden cuando la lee.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Effect {
    /// Abre una pagina: lo que venga detras pertenece a ella.
    OpensPage,
    /// Manda a otro sitio nada mas entrar, sin que nadie toque nada.
    OnEnter,
    /// Se pinta como un control que el jugador dispara.
    Control,
}

#[derive(Debug, Clone, Copy)]
pub struct Spec {
    /// Como se escribe.
    pub verb: &'static str,
    /// Otras formas de escribirlo que tambien valen.
    pub aliases: &'static [&'static str],
    pub target: Target,
    pub effect: Effect,
    pub params: &'static [Param],
    /// Una linea de ayuda, para el editor de sitios.
    pub help: &'static str,
}

/// Todas las ordenes que existen.
pub const CATALOG: &[Spec] = &[
    Spec {
        verb: "page",
        aliases: &[],
        target: Target::Ident,
        effect: Effect::OpensPage,
        params: &[],
        help: "Abre una pagina del sitio. Lo que se escriba debajo pertenece a ella",
    },
    Spec {
        verb: "link",
        aliases: &[],
        target: Target::Ident,
        effect: Effect::Control,
        params: &[],
        help: "Enlace a otra pagina del mismo sitio",
    },
    Spec {
        verb: "go_to",
        aliases: &["goto"],
        target: Target::Ip,
        effect: Effect::Control,
        params: &[],
        help: "Enlace a otra maquina por su direccion",
    },
    Spec {
        verb: "redirect",
        aliases: &[],
        target: Target::Ip,
        effect: Effect::OnEnter,
        params: &[],
        help: "Manda a otra maquina nada mas entrar en el sitio",
    },
    // Las dos de abajo hablan con el banco. La pagina solo declara la intencion: quien pide
    // el PIN y llama a la API es el navegador del juego, nunca el sitio. Un sitio que
    // pudiera cobrar solo por pintarse no seria una estafa, seria un robo automatico.
    Spec {
        verb: "transfer",
        aliases: &[],
        target: Target::Account,
        effect: Effect::Control,
        params: &[
            Param { name: "amount", required: true, numeric: true },
            Param { name: "concept", required: false, numeric: false },
        ],
        help: "Ofrece pagar a una cuenta. El importe va en `amount`",
    },
    Spec {
        verb: "charge",
        aliases: &[],
        target: Target::Account,
        effect: Effect::Control,
        params: &[
            Param { name: "amount", required: true, numeric: true },
            Param { name: "concept", required: false, numeric: false },
        ],
        help: "Emite un cobro contra quien visita, a favor de la cuenta indicada",
    },
];

/// Busca una orden por como se haya escrito.
pub fn find(verb: &str) -> Option<&'static Spec> {
    let verb = verb.trim().to_lowercase();

    CATALOG
        .iter()
        .find(|spec| spec.verb == verb || spec.aliases.contains(&verb.as_str()))
}

/// Las ordenes que existen, para decirlo cuando alguien escribe una que no.
pub fn verbs() -> String {
    CATALOG
        .iter()
        .map(|spec| format!("`{}`", spec.verb))
        .collect::<Vec<_>>()
        .join(", ")
}

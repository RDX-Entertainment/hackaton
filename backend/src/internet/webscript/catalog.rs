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
    /// La orden no lleva destino: lo que necesita saber lo pregunta el navegador, o sale de
    /// la propia maquina que sirve la pagina.
    None,
}

/// De que tiene que tener pinta el valor de un parametro.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// Texto corriente.
    Text,
    /// Un entero mayor que cero.
    Number,
    /// Un nombre corto, como el de una pagina.
    Ident,
    /// Un color, `#fcd535`.
    Color,
}

/// Un parametro con nombre, de los que van detras del segundo `|`.
#[derive(Debug, Clone, Copy)]
pub struct Param {
    pub name: &'static str,
    pub required: bool,
    pub kind: Kind,
}

/// De que familia es una orden, que es lo que decide como se concede.
///
/// Las **normales** las usa cualquier sitio. Las **integradas** llevan la interfaz dentro
/// del HTML —campos propios donde se teclea, huecos que el juego rellena— y eso es un
/// privilegio: una pagina que pide un PIN con su propio aspecto es justo lo que haria una
/// falsa. Quien tiene cada una no se decide aqui sino en `action_grants`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Family {
    Normal,
    Integrated,
}

impl Family {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Integrated => "integrated",
        }
    }
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
    pub family: Family,
    pub effect: Effect,
    pub params: &'static [Param],
    /// Una linea de ayuda, para el editor de sitios.
    pub help: &'static str,
}

/// A donde ir cuando la operacion sale bien. Lo llevan todas las de banca: entrar y quedarse
/// en la pantalla de entrar seria dejar al jugador donde ya estaba.
const THEN: Param = Param {
    name: "then",
    required: false,
    kind: Kind::Ident,
};

/// Lo que una pagina autorizada puede pedir que se teclee, marcandolo con `data-field`.
///
/// Es una lista cerrada a proposito. Un banco pide una cuenta y un PIN porque el juego sabe
/// que hacer con eso; no puede inventarse un campo y esperar que alguien lo lea.
pub const FIELDS: &[&str] = &["account", "pin", "to_account", "amount", "concept"];

/// Los huecos que el juego rellena, marcados con `data-bind`. El banco decide donde van y
/// con que aspecto; lo que se escribe dentro lo pone el juego.
pub const BINDS: &[&str] = &[
    "bank", "account", "holder", "balance", "status", "amount", "concept", "other_account",
];

/// Cuando se ve un elemento, marcado con `data-when`. `in` solo con sesion abierta, `out`
/// solo sin ella: es lo que deja que una barra de navegacion cambie sin recargar el sitio.
pub const WHENS: &[&str] = &["in", "out"];

/// Las listas que el juego repite, marcadas con `data-list`. Dentro va una plantilla con los
/// `data-bind` de cada fila.
pub const LISTS: &[&str] = &["movements"];

/// Todas las ordenes que existen.
pub const CATALOG: &[Spec] = &[
    Spec {
        verb: "page",
        aliases: &[],
        target: Target::Ident,
        family: Family::Normal,
        effect: Effect::OpensPage,
        params: &[],
        help: "Abre una pagina del sitio. Lo que se escriba debajo pertenece a ella",
    },
    Spec {
        verb: "link",
        aliases: &[],
        target: Target::Ident,
        family: Family::Normal,
        effect: Effect::Control,
        params: &[],
        help: "Enlace a otra pagina del mismo sitio",
    },
    Spec {
        verb: "go_to",
        aliases: &["goto"],
        target: Target::Ip,
        family: Family::Normal,
        effect: Effect::Control,
        params: &[],
        help: "Enlace a otra maquina por su direccion",
    },
    Spec {
        verb: "redirect",
        aliases: &[],
        target: Target::Ip,
        family: Family::Normal,
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
        family: Family::Normal,
        effect: Effect::Control,
        params: &[
            Param { name: "amount", required: true, kind: Kind::Number },
            Param { name: "concept", required: false, kind: Kind::Text },
        ],
        help: "Ofrece pagar a una cuenta. El importe va en `amount`",
    },
    // La banca, una orden por operacion.
    //
    // Estas son **las que solo sirven en la web de un banco autorizado**, y no porque nadie
    // las prohiba: el navegador solo tiene un banco al que llamar cuando el servidor le dice
    // cual es, y eso solo pasa si la maquina pertenece a una organizacion con `banking`. En
    // cualquier otra pagina el boton no hace nada.
    //
    // Lo que las separa de `transfer` y `charge` es donde se teclea. Un banco autorizado
    // pide las credenciales **en su propio HTML**, con sus `data-field` y su CSS; una tienda
    // no puede, y su cobro lo atiende el juego con su propio aspecto. Ese es el privilegio:
    // no es poder cobrar, es poder parecer un banco.
    Spec {
        verb: "bank_login",
        aliases: &[],
        target: Target::None,
        family: Family::Integrated,
        effect: Effect::Control,
        params: &[THEN],
        help: "Entra con los campos `account` y `pin` de la pagina",
    },
    Spec {
        verb: "bank_open",
        aliases: &[],
        target: Target::None,
        family: Family::Integrated,
        effect: Effect::Control,
        params: &[THEN],
        help: "Abre una cuenta con el `pin` que se haya tecleado",
    },
    Spec {
        verb: "bank_transfer",
        aliases: &[],
        target: Target::None,
        family: Family::Integrated,
        effect: Effect::Control,
        params: &[THEN],
        help: "Envia desde la cuenta abierta, con `to_account`, `amount` y `concept`",
    },
    Spec {
        verb: "bank_statement",
        aliases: &[],
        target: Target::None,
        family: Family::Integrated,
        effect: Effect::Control,
        params: &[THEN],
        help: "Rellena la lista `movements` con los movimientos de la cuenta",
    },
    Spec {
        verb: "bank_logout",
        aliases: &[],
        target: Target::None,
        family: Family::Integrated,
        effect: Effect::Control,
        params: &[THEN],
        help: "Cierra la sesion de banca",
    },
    Spec {
        verb: "charge",
        aliases: &[],
        target: Target::Account,
        family: Family::Normal,
        effect: Effect::Control,
        params: &[
            Param { name: "amount", required: true, kind: Kind::Number },
            Param { name: "concept", required: false, kind: Kind::Text },
        ],
        help: "Emite un cobro contra quien visita, a favor de la cuenta indicada",
    },
];

/// Las ordenes de una familia, por su verbo.
pub fn of_family(family: Family) -> Vec<&'static str> {
    CATALOG
        .iter()
        .filter(|spec| spec.family == family)
        .map(|spec| spec.verb)
        .collect()
}

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

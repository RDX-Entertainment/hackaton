//! Lo que el tutorial sabe decir: los hechos que cierran un paso y las acciones que puede
//! permitir o negar.
//!
//! Es un vocabulario **cerrado**, y esa es la decision. El panel no deja escribir una
//! consulta ni el nombre de un endpoint: elige de estas listas. Un codigo que no este aqui no
//! lo sabe comprobar nadie, asi que dejar escribirlo seria ofrecer un boton que no hace nada
//! y descubrirlo tres semanas despues, con un jugador atascado.
//!
//! Las etiquetas van en castellano porque su unico lector es el panel de administracion, que
//! esta en castellano. Lo que viaja al jugador son los codigos, que traduce el cliente.

/// Los hechos del mundo que pueden cerrar un paso.
///
/// Cada uno tiene su `EXISTS` en `repo::facts` y su valor permitido en el `CHECK` de la
/// tabla: anadir uno son tres lineas, en tres sitios que se leen juntos.
pub const FACTS: &[(&str, &str)] = &[
    ("modules", "Ha investigado alguna pieza"),
    ("blueprints", "Ha guardado algun diseno"),
    ("programs", "Tiene algun programa compilado"),
    ("running", "Tiene algo cargado en memoria"),
    ("intrusions", "Ha entrado en alguna maquina"),
];

/// Las acciones que el tutorial puede dejar hacer o negar.
///
/// No estan todas las del juego, y no es un olvido: aqui solo entra lo que un tutorial tiene
/// motivo para gobernar. Cada codigo lo comprueba un sitio concreto del servidor --el que
/// llama a `guard`-- y anadir uno sin ese sitio seria escribir una promesa.
pub const ACTIONS: &[(&str, &str)] = &[
    ("lab.investigar", "Lab · investigar una pieza"),
    ("lab.disenar", "Lab · guardar un diseno"),
    ("lab.compilar", "Lab · compilar un diseno"),
    ("lab.mejorar", "Lab · ajustar un programa"),
    ("lab.lanzar", "Lab · lanzar un programa contra una maquina"),
    ("discos.crear", "Discos · crear un fichero"),
    ("discos.cargar", "Discos · cargar en memoria"),
    ("discos.descargar", "Discos · sacar de memoria"),
    ("discos.ocultar", "Discos · ocultar o revelar"),
    ("discos.mover", "Discos · mover de sitio"),
    ("red.entrar", "Red · entrar en una maquina"),
    ("red.hackear", "Red · romper una contrasena"),
    ("red.analizar", "Red · analizar una maquina"),
    ("banco.transferir", "Banco · transferir dinero"),
    ("encargo.aceptar", "Encargos · aceptar un encargo"),
];

/// La accion del tutorial que corresponde a una de las numeradas del legado.
///
/// Las tareas se encolan por numero --`13` es cargar en memoria, `11` es romper una
/// contrasena-- y ese numero es lo unico que llega al servidor cuando el jugador pulsa. Sin
/// esta traduccion, vetar «cargar en memoria» obligaria a hablarle al panel de numeros de un
/// PHP de hace quince anos.
///
/// Lo que no esta aqui no lo gobierna el tutorial: se deja pasar. Negar por omision
/// convertiria cada accion nueva del juego en un fallo silencioso durante el tutorial.
pub fn action_of_process(action: u8) -> Option<&'static str> {
    match action {
        4 | 5 => Some("discos.ocultar"),
        11 => Some("red.hackear"),
        13 => Some("discos.cargar"),
        14 => Some("discos.descargar"),
        17 => Some("lab.investigar"),
        22 | 23 => Some("red.analizar"),
        _ => None,
    }
}

/// Si el codigo existe. Lo usa el panel al guardar: una lista con una errata es un bloqueo
/// que no se levanta nunca, porque el jugador no puede hacer una accion que no existe.
pub fn is_action(code: &str) -> bool {
    ACTIONS.iter().any(|(known, _)| *known == code)
}

pub fn is_fact(code: &str) -> bool {
    FACTS.iter().any(|(known, _)| *known == code)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Las cinco preguntas de A3 siguen siendo las que la tabla admite en su `CHECK`.
    #[test]
    fn the_seeded_facts_are_the_ones_the_world_answers() {
        for fact in ["modules", "blueprints", "programs", "running", "intrusions"] {
            assert!(is_fact(fact), "{fact} deberia estar en el vocabulario");
        }
    }

    /// Cargar en memoria es la accion 13 del legado, y es la que el cuarto paso deja hacer.
    #[test]
    fn loading_into_memory_has_a_name() {
        assert_eq!(action_of_process(13), Some("discos.cargar"));
        assert!(is_action("discos.cargar"));
    }

    /// Lo que el tutorial no gobierna se deja pasar, y por eso devuelve `None` en vez de un
    /// codigo inventado.
    #[test]
    fn an_action_nobody_governs_is_not_invented() {
        assert_eq!(action_of_process(27), None);
    }
}

//! Que se puede escribir en la pagina de una maquina.
//!
//! El cliente ya pinta estos sitios en un marco con `sandbox` y sin permisos, asi que
//! aunque llegara un `<script>` no se ejecutaria. Eso no basta: el visor es una defensa
//! prestada. Si manana alguien pinta el mismo HTML en otro sitio -- una vista previa mal
//! hecha, un correo, una exportacion -- el guion viajaria con el.
//!
//! Por eso la regla se aplica al escribir. Lo que hay guardado en la base es lo que se
//! puede servir, y no depende de quien lo mire.
//!
//! Un sitio es texto, estructura, CSS y las ordenes del juego. Nada mas:
//!
//!   * SIN GUIONES     ni etiqueta, ni manejadores en linea, ni URLs `javascript:`
//!   * SIN CARGAS      nada que traiga un recurso de fuera
//!
//! Las ordenes del juego viven en `<action>` y no en `<script>`, que sigue prohibido. Asi
//! no hay que debilitar ninguna regla para que un sitio pueda navegar: lo que hay dentro de
//! un `<action>` no es codigo sino una lista de destinos, y lo comprueba [`super::webscript`].
//!
//! Lo segundo no es paranoia sobrante: una imagen alojada fuera es un contador de
//! visitas. En un juego donde la direccion IP es la moneda, poner un pixel en tu pagina
//! seria registrar la de todo el que pase, sin hackear nada.

use crate::error::ApiError;

/// Doce mil caracteres dan para una pagina con su hoja dentro y no para un vertedero.
pub const MAX_LENGTH: usize = 16_384;

/// Etiquetas que no entran. `<style>` no esta aqui a proposito: es el punto. `<action>`
/// tampoco, pero eso no es un hueco: lo que lleva dentro lo revisa `webscript`.
const FORBIDDEN_TAGS: [&str; 10] = [
    "script", "iframe", "object", "embed", "applet", "form", "base", "link", "meta",
    "noscript",
];

/// Lo que traeria algo de fuera.
const FORBIDDEN_LOADS: [&str; 5] = ["src=", "@import", "url(", "srcset=", "background="];

const FORBIDDEN_SCHEMES: [&str; 3] = ["javascript:", "vbscript:", "data:text/html"];

pub fn check(html: &str) -> Result<(), ApiError> {
    if html.len() > MAX_LENGTH {
        return Err(ApiError::BadRequest(format!(
            "TOO_LONG: la pagina ocupa {} caracteres y el maximo es {MAX_LENGTH}",
            html.len()
        )));
    }

    let lower = html.to_lowercase();

    for tag in FORBIDDEN_TAGS {
        if lower.contains(&format!("<{tag}")) {
            return Err(ApiError::BadRequest(format!(
                "TAG_NOT_ALLOWED: <{tag}> no se permite. Un sitio es texto, estructura y CSS"
            )));
        }
    }

    for scheme in FORBIDDEN_SCHEMES {
        if lower.contains(scheme) {
            return Err(ApiError::BadRequest(format!(
                "SCHEME_NOT_ALLOWED: {scheme} no se permite"
            )));
        }
    }

    for load in FORBIDDEN_LOADS {
        if lower.contains(load) {
            return Err(ApiError::BadRequest(format!(
                "EXTERNAL_NOT_ALLOWED: {load} traeria un recurso de fuera, y eso convierte \
                 la pagina en un contador de visitas. El sitio tiene que bastarse solo"
            )));
        }
    }

    if let Some(handler) = find_handler(&lower) {
        return Err(ApiError::BadRequest(format!(
            "HANDLER_NOT_ALLOWED: {handler} es un manejador de eventos, y eso es codigo"
        )));
    }

    // Lo ultimo, cuando ya se sabe que no hay nada peligroso suelto: que lo que haya dentro
    // de un `<action>` sean ordenes del juego y no otra cosa.
    super::webscript::check(html)?;

    Ok(())
}

/// Busca un atributo `on...=`, que es como se cuela un guion sin escribir `<script>`.
///
/// A mano y no con una expresion regular porque es la unica que haria falta en todo el
/// proyecto, y la regla cabe en veinte lineas: un `on` que empieza atributo, letras
/// detras, y un `=` despues de los espacios que haya.
fn find_handler(lower: &str) -> Option<String> {
    let bytes = lower.as_bytes();

    for (start, _) in lower.match_indices("on") {
        // Tiene que empezar un atributo: antes va un espacio o el signo de la etiqueta.
        let opens = match start.checked_sub(1).map(|before| bytes[before]) {
            None => false,
            Some(before) => before.is_ascii_whitespace() || before == b'<' || before == b'"'
                || before == b'\'' || before == b'/',
        };

        if !opens {
            continue;
        }

        let mut at = start + 2;

        while at < bytes.len() && bytes[at].is_ascii_alphabetic() {
            at += 1;
        }

        // `on` a secas no es un manejador; `onclick` si.
        if at == start + 2 {
            continue;
        }

        let name = &lower[start..at];

        while at < bytes.len() && bytes[at].is_ascii_whitespace() {
            at += 1;
        }

        if bytes.get(at) == Some(&b'=') {
            return Some(name.to_string());
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rejects(html: &str) -> bool {
        check(html).is_err()
    }

    #[test]
    fn allows_a_page_with_its_own_stylesheet() {
        assert!(check("<style>body{color:red}</style><h1>Hola</h1>").is_ok());
    }

    #[test]
    fn rejects_scripts_however_they_are_written() {
        assert!(rejects("<script>alert(1)</script>"));
        assert!(rejects("<SCRIPT>alert(1)</SCRIPT>"));
        assert!(rejects("<a href=\"javascript:alert(1)\">x</a>"));
    }

    #[test]
    fn rejects_inline_handlers() {
        assert!(rejects("<div onclick=\"x()\">hola</div>"));
        assert!(rejects("<div ONMOUSEOVER = 'x()'>hola</div>"));
        assert!(rejects("<div\nonload=x>hola</div>"));
    }

    #[test]
    fn a_word_starting_with_on_is_not_a_handler() {
        assert!(check("<p>one two only online</p>").is_ok());
        assert!(check("<style>.only{color:red}</style>").is_ok());
    }

    #[test]
    fn rejects_anything_loaded_from_outside() {
        assert!(rejects("<img src=\"http://tracker.example/pixel.gif\">"));
        assert!(rejects("<style>@import url(http://x.example/a.css)</style>"));
        assert!(rejects("<style>body{background:url(http://x.example/a.png)}</style>"));
    }

    #[test]
    fn rejects_a_page_that_is_too_long() {
        assert!(rejects(&"a".repeat(MAX_LENGTH + 1)));
    }
}

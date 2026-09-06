//! Que puede llevar la hoja de estilo de un sitio.
//!
//! La regla es una sola y es la misma que en el HTML: **nada que traiga un recurso de
//! fuera**. En un juego donde la direccion IP es la moneda, una imagen alojada en otro
//! servidor es un contador de visitas: pones un fondo en tu pagina y registras la IP de
//! todo el que pase, sin hackear nada.
//!
//! Se comprueba tokenizando, no buscando texto. La version anterior rechazaba cualquier
//! pagina que contuviera la cadena `url(`, y eso tumbaba un parrafo que mencionara `url(`
//! escribiendo sobre urls, sin que cargara nada. El tokenizador distingue las dos cosas:
//! una es una funcion y la otra es una frase.

use cssparser::{ParseError, ParseErrorKind, Parser, ParserInput, Token};

/// Funciones que terminan pidiendole algo a la red. `url()` es la evidente; el resto son
/// las que hacen lo mismo con otro nombre.
const FORBIDDEN_FUNCTIONS: [&str; 6] = [
    "url",
    "src",
    "image",
    "image-set",
    "-webkit-image-set",
    "expression",
];

/// Reglas `@` que traen otra hoja entera.
const FORBIDDEN_AT_RULES: [&str; 2] = ["import", "namespace"];

/// Revisa una hoja de estilo, o el valor de un atributo `style`.
///
/// `where_at` sale en el mensaje para que el autor sepa donde mirar cuando la pagina tiene
/// una hoja larga y un par de estilos sueltos.
pub fn check(css: &str, where_at: &str) -> Result<(), String> {
    let mut input = ParserInput::new(css);
    let mut parser = Parser::new(&mut input);

    match walk(&mut parser, where_at) {
        Ok(()) => Ok(()),

        Err(ParseError {
            kind: ParseErrorKind::Custom(message),
            ..
        }) => Err(message),

        // Un CSS que ni siquiera se deja tokenizar no es un ataque, es una errata. Pasa: el
        // navegador tambien se salta lo que no entiende, y el autor lo vera al mirar.
        Err(_) => Ok(()),
    }
}

fn walk<'i>(parser: &mut Parser<'i, '_>, where_at: &str) -> Result<(), ParseError<'i, String>> {
    // `url (x)`, con un espacio, no es un token de url: el tokenizador ve el nombre suelto y
    // luego un parentesis, y asi escrito el navegador tampoco carga nada. Se rechaza igual.
    // Nadie escribe `url` seguido de un parentesis queriendo decir otra cosa, y no depender
    // de esa finura vale mas que las cuatro lineas que cuesta.
    let mut pending = None;

    loop {
        let token = match parser.next_including_whitespace_and_comments() {
            Ok(token) => token.clone(),
            Err(_) => return Ok(()),
        };

        if let Token::WhiteSpace(_) | Token::Comment(_) = token {
            continue;
        }

        let named = pending.take();

        if let Token::Ident(ref name) = token {
            let name = name.to_lowercase();

            if FORBIDDEN_FUNCTIONS.contains(&name.as_str()) {
                pending = Some(name);
            }
        }

        if let (Token::ParenthesisBlock, Some(name)) = (&token, named) {
            return Err(parser.new_custom_error(format!(
                "EXTERNAL_NOT_ALLOWED: {where_at} usa `{name} (...)`, y eso traeria un \
                 recurso de fuera. El sitio tiene que bastarse solo"
            )));
        }

        match token {
            // `url(imagen.png)` sin comillas: el tokenizador lo reconoce como tal.
            Token::UnquotedUrl(_) | Token::BadUrl(_) => {
                return Err(parser.new_custom_error(format!(
                    "EXTERNAL_NOT_ALLOWED: {where_at} usa `url(...)`, y eso traeria un \
                     recurso de fuera. El sitio tiene que bastarse solo"
                )));
            }

            Token::AtKeyword(ref name) => {
                let name = name.to_lowercase();

                if FORBIDDEN_AT_RULES.contains(&name.as_str()) {
                    return Err(parser.new_custom_error(format!(
                        "EXTERNAL_NOT_ALLOWED: {where_at} usa `@{name}`, y eso traeria otra \
                         hoja de estilo de fuera. El sitio tiene que bastarse solo"
                    )));
                }
            }

            // `url("imagen.png")` con comillas se tokeniza como funcion, no como url.
            Token::Function(ref name) => {
                let name = name.to_lowercase();

                if FORBIDDEN_FUNCTIONS.contains(&name.as_str()) {
                    return Err(parser.new_custom_error(format!(
                        "EXTERNAL_NOT_ALLOWED: {where_at} usa `{name}(...)`, y eso traeria un \
                         recurso de fuera. El sitio tiene que bastarse solo"
                    )));
                }

                parser.parse_nested_block(|nested| walk(nested, where_at))?;
            }

            Token::ParenthesisBlock | Token::SquareBracketBlock | Token::CurlyBracketBlock => {
                parser.parse_nested_block(|nested| walk(nested, where_at))?;
            }

            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rejects(css: &str) -> bool {
        check(css, "la hoja").is_err()
    }

    #[test]
    fn una_hoja_normal_pasa() {
        assert!(check("body{margin:0;background:#12100c;color:#e8dfc8}", "la hoja").is_ok());
        assert!(check("h1{font:26px Georgia,serif;letter-spacing:.5px}", "la hoja").is_ok());
    }

    #[test]
    fn rechaza_lo_que_carga_de_fuera() {
        assert!(rejects("body{background:url(http://x.example/a.png)}"));
        assert!(rejects("body{background:url('http://x.example/a.png')}"));
        assert!(rejects("@import url(http://x.example/a.css);"));
        assert!(rejects("@import 'http://x.example/a.css';"));
    }

    #[test]
    fn ni_el_espacio_ni_las_mayusculas_lo_esquivan() {
        // Escrito con espacio el navegador tampoco lo carga —deja de ser una funcion—, pero
        // no se depende de esa finura: si parece que carga, se rechaza.
        assert!(rejects("body{background:url ('http://x.example/a.png')}"));
        assert!(rejects("body{background:URL(http://x.example/a.png)}"));
    }

    #[test]
    fn tambien_dentro_de_una_media_query() {
        assert!(rejects("@media (min-width:40rem){body{background:url(x.png)}}"));
    }

    #[test]
    fn hablar_de_urls_no_es_cargar_una() {
        // Esto lo rechazaba la version anterior, y no cargaba nada de nada.
        assert!(check("body::after{content:'escribe url(...) para enlazar'}", "la hoja").is_ok());
    }

    #[test]
    fn un_css_roto_no_es_un_ataque() {
        assert!(check("body{color:", "la hoja").is_ok());
    }
}

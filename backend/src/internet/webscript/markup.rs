//! Los controles que van escritos en el propio HTML.
//!
//! Un bloque `<action>` sirve para la estructura —abrir paginas, mandar a otra maquina—,
//! pero los botones no: un boton tiene que poder estar donde el autor quiera y llevar el
//! estilo del sitio. Por eso se escriben como HTML normal y se marcan con atributos:
//!
//! ```html
//! <button data-action="link" data-target="cuotas">Ver cuotas</button>
//! <a data-action="go_to" data-target="45.123.10.5">Nuestra sede</a>
//! <button data-action="transfer" data-target="349463068" data-amount="2500">Pagar</button>
//! ```
//!
//! El navegador del juego los reconoce por esos atributos y es el quien obedece: el sitio
//! declara, nunca ejecuta. Las ordenes son las mismas de [`super::catalog`], asi que una
//! accion nueva vale en los dos sitios sin escribir nada aqui.
//!
//! `data-*` no es un manejador de eventos: `on...=` sigue prohibido, y el HTML del autor
//! sigue sin poder traer una linea de codigo.

use super::catalog::{self, Effect, Target};

/// Recorre los `data-action` del HTML y devuelve el motivo del primero que este mal.
pub fn check(html: &str) -> Result<(), String> {
    for tag in tags_with_actions(html) {
        let Some(verb) = attribute(&tag, "data-action") else {
            continue;
        };

        let Some(spec) = catalog::find(&verb) else {
            return Err(format!(
                "ACTION_UNKNOWN: `{verb}` no es una orden. Hay {}",
                catalog::verbs()
            ));
        };

        if spec.effect != Effect::Control {
            return Err(format!(
                "ACTION_PLACE: `{}` no es un boton, va dentro de un bloque <action>",
                spec.verb
            ));
        }

        let target = match attribute(&tag, "data-target") {
            Some(target) => target,
            None if spec.target == Target::None => String::new(),
            None => {
                return Err(format!(
                    "ACTION_TARGET: al boton de `{}` le falta `data-target`. {}",
                    spec.verb, spec.help
                ));
            }
        };

        super::parser::check_target(spec, &target)?;

        for declared in spec.params {
            let name = format!("data-{}", declared.name);
            let value = attribute(&tag, &name);

            match value {
                None if declared.required => {
                    return Err(format!(
                        "ACTION_PARAM: al boton de `{}` le falta `{name}`. {}",
                        spec.verb, spec.help
                    ));
                }
                Some(ref value) => {
                    super::parser::check_value(&name, declared.kind, value)?;
                }
                _ => {}
            }
        }
    }

    check_named(html, "data-field", catalog::FIELDS, "se puede teclear")?;
    check_named(html, "data-bind", catalog::BINDS, "rellena el juego")?;
    check_named(html, "data-list", catalog::LISTS, "repite el juego")?;
    check_named(html, "data-when", catalog::WHENS, "decide cuando se ve")?;

    Ok(())
}

/// Que los `data-field`, `data-bind` y `data-list` sean de los que el juego conoce.
///
/// Sin esto, escribir `data-bind="saldo"` en vez de `balance` dejaria un hueco que no se
/// rellena nunca y sin una sola pista de por que.
fn check_named(
    html: &str,
    attribute_name: &str,
    known: &[&str],
    what: &str,
) -> Result<(), String> {
    for tag in tags_with(html, attribute_name) {
        let Some(value) = attribute(&tag, attribute_name) else {
            continue;
        };

        if !known.contains(&value.to_lowercase().as_str()) {
            return Err(format!(
                "ACTION_FIELD: `{attribute_name}=\"{value}\"` no existe. Lo que {what} es: {}",
                known.join(", ")
            ));
        }
    }

    Ok(())
}

/// Las etiquetas que llevan un `data-action`, cada una entera para poder leerle el resto.
fn tags_with_actions(html: &str) -> Vec<String> {
    tags_with(html, "data-action")
}

fn tags_with(html: &str, attribute_name: &str) -> Vec<String> {
    let lower = html.to_lowercase();
    let mut found = Vec::new();

    for (at, _) in lower.match_indices(attribute_name) {
        // Retroceder hasta el `<` que abre la etiqueta y avanzar hasta el `>` que la cierra.
        let Some(open) = lower[..at].rfind('<') else {
            continue;
        };

        let end = lower[at..].find('>').map_or(html.len(), |to| at + to);

        found.push(html[open..end].to_owned());
    }

    found
}

/// El valor de un atributo dentro de una etiqueta. Admite comillas dobles y simples.
fn attribute(tag: &str, name: &str) -> Option<String> {
    let lower = tag.to_lowercase();
    let at = lower.find(&format!("{name}="))?;
    let rest = &tag[at + name.len() + 1..];

    let quote = rest.chars().next()?;

    if quote != '"' && quote != '\'' {
        return None;
    }

    let value = &rest[1..];
    let end = value.find(quote)?;

    Some(value[..end].trim().to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn acepta_un_boton_bien_escrito() {
        assert!(check(r#"<button data-action="link" data-target="precios">Ver</button>"#).is_ok());
        assert!(check(r#"<a data-action="go_to" data-target="45.123.10.5">Ir</a>"#).is_ok());
    }

    #[test]
    fn acepta_un_pago_con_su_importe() {
        let html = r#"<button data-action="transfer" data-target="349463068"
                      data-amount="2500">Pagar</button>"#;

        assert!(check(html).is_ok());
    }

    #[test]
    fn un_html_sin_botones_no_tiene_nada_que_revisar() {
        assert!(check("<p>Hola</p>").is_ok());
    }

    #[test]
    fn rechaza_una_orden_que_no_existe() {
        assert!(check(r#"<button data-action="hackear" data-target="x">x</button>"#).is_err());
    }

    #[test]
    fn rechaza_un_boton_sin_destino() {
        let error = check(r#"<button data-action="go_to">Ir</button>"#).unwrap_err();

        assert!(error.contains("data-target"));
    }

    #[test]
    fn rechaza_una_ip_que_no_lo_es() {
        assert!(check(r#"<a data-action="go_to" data-target="fuera">x</a>"#).is_err());
    }

    #[test]
    fn rechaza_un_pago_sin_importe() {
        let error =
            check(r#"<button data-action="transfer" data-target="349463068">x</button>"#)
                .unwrap_err();

        assert!(error.contains("data-amount"));
    }

    #[test]
    fn rechaza_un_importe_que_no_es_numero() {
        let html = r#"<button data-action="transfer" data-target="349463068"
                      data-amount="gratis">x</button>"#;

        assert!(check(html).is_err());
    }

    #[test]
    fn una_pagina_no_se_abre_desde_un_boton() {
        let error = check(r#"<button data-action="page" data-target="uno">x</button>"#)
            .unwrap_err();

        assert!(error.contains("ACTION_PLACE"));
    }

    #[test]
    fn lee_atributos_con_comillas_simples() {
        assert!(check("<button data-action='link' data-target='uno'>x</button>").is_ok());
    }
}

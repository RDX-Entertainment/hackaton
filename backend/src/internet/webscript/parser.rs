//! Como se lee una linea y como se trocea el HTML.
//!
//! Nada de aqui sabe que ordenes existen: eso lo dice [`super::catalog`]. Este fichero solo
//! sabe que una orden se escribe `verbo: destino | texto | clave=valor` y como partir el
//! HTML por los bloques `<action>`.

use std::collections::BTreeMap;

use super::catalog::{self, Effect, Kind, Spec, Target};

/// Una orden ya leida, todavia sin juzgar si es valida.
pub struct Line {
    pub spec: &'static Spec,
    pub target: String,
    pub label: String,
    pub params: BTreeMap<String, String>,
}

/// Un trozo del sitio: o HTML, o un bloque de ordenes.
pub enum Chunk<'a> {
    Html(&'a str),
    Actions(&'a str),
}

/// Parte el HTML en trozos alternos de contenido y ordenes.
///
/// Un bloque sin cerrar corta el recorrido y lo que quede se trata como HTML: al servir no
/// se puede fallar, y quien avisa de eso es la comprobacion de guardado.
pub fn split(html: &str) -> Vec<Chunk<'_>> {
    let mut chunks = Vec::new();
    let mut rest = html;

    while let Some(open) = find_tag(rest, "<action") {
        let (head, from_open) = rest.split_at(open);

        let Some(body_start) = from_open.find('>') else {
            break;
        };

        let body_and_tail = &from_open[body_start + 1..];

        let Some(close) = find_tag(body_and_tail, "</action") else {
            break;
        };

        if !head.is_empty() {
            chunks.push(Chunk::Html(head));
        }

        chunks.push(Chunk::Actions(&body_and_tail[..close]));

        let tail = &body_and_tail[close..];
        rest = tail.find('>').map_or("", |end| &tail[end + 1..]);
    }

    if !rest.is_empty() {
        chunks.push(Chunk::Html(rest));
    }

    chunks
}

/// Lee una linea. Devuelve `None` si esta vacia o es un comentario, y `Err` con el motivo
/// si esta escrita de una forma que no se entiende.
pub fn read(line: &str) -> Option<Result<Line, String>> {
    let clean = strip_comment(line);

    if clean.is_empty() {
        return None;
    }

    Some(read_clean(clean))
}

fn read_clean(clean: &str) -> Result<Line, String> {
    let Some((verb, rest)) = clean.split_once(':') else {
        return Err(format!(
            "ACTION_SYNTAX: «{clean}» no es una orden. Se escriben `orden: destino | texto`"
        ));
    };

    let Some(spec) = catalog::find(verb) else {
        return Err(format!(
            "ACTION_UNKNOWN: `{}` no es una orden. Hay {}",
            verb.trim(),
            catalog::verbs()
        ));
    };

    let mut pieces = rest.split('|');
    let target = pieces.next().unwrap_or("").trim().to_owned();
    let label = pieces.next().unwrap_or("").trim().to_owned();
    let params = read_params(pieces.collect::<Vec<_>>().join("|").trim())?;

    if target.is_empty() && spec.target != Target::None {
        return Err(format!(
            "ACTION_TARGET: `{}` necesita un destino. {}",
            spec.verb, spec.help
        ));
    }

    check_target(spec, &target)?;
    check_params(spec, &params)?;

    Ok(Line {
        spec,
        label: if label.is_empty() { target.clone() } else { label },
        target: match spec.target {
            Target::Ident => target.to_lowercase(),
            _ => target,
        },
        params,
    })
}

fn read_params(text: &str) -> Result<BTreeMap<String, String>, String> {
    let mut params = BTreeMap::new();

    if text.is_empty() {
        return Ok(params);
    }

    for piece in text.split_whitespace() {
        let Some((name, value)) = piece.split_once('=') else {
            return Err(format!(
                "ACTION_PARAM: «{piece}» no es un parametro. Se escriben `clave=valor`"
            ));
        };

        params.insert(name.trim().to_lowercase(), value.trim().to_owned());
    }

    Ok(params)
}

pub fn check_target(spec: &Spec, target: &str) -> Result<(), String> {
    match spec.target {
        Target::None => Ok(()),
        Target::Ip => parse_ip(target).map(|_| ()).ok_or_else(|| {
            format!(
                "ACTION_IP: «{target}» no es una direccion. Se escriben como 45.123.10.5"
            )
        }),
        Target::Ident => {
            let ok = target.chars().count() <= MAX_IDENT
                && target
                    .chars()
                    .all(|one| one.is_ascii_alphanumeric() || one == '-' || one == '_');

            ok.then_some(()).ok_or_else(|| {
                format!(
                    "ACTION_ID: «{target}» no vale como nombre. Letras, numeros, guion y \
                     guion bajo, hasta {MAX_IDENT}"
                )
            })
        }
        Target::Account => {
            let ok = target.len() == ACCOUNT_DIGITS
                && target.chars().all(|one| one.is_ascii_digit());

            ok.then_some(()).ok_or_else(|| {
                format!(
                    "ACTION_ACCOUNT: «{target}» no es una cuenta. Son {ACCOUNT_DIGITS} digitos"
                )
            })
        }
    }
}

fn check_params(
    spec: &Spec,
    params: &BTreeMap<String, String>,
) -> Result<(), String> {
    for declared in spec.params {
        match params.get(declared.name) {
            None if declared.required => {
                return Err(format!(
                    "ACTION_PARAM: a `{}` le falta `{}=`. {}",
                    spec.verb, declared.name, spec.help
                ));
            }
            Some(value) => check_value(declared.name, declared.kind, value)?,
            _ => {}
        }
    }

    for name in params.keys() {
        if !spec.params.iter().any(|declared| declared.name == name) {
            return Err(format!(
                "ACTION_PARAM: `{}` no admite `{name}=`. {}",
                spec.verb, spec.help
            ));
        }
    }

    Ok(())
}

/// Comprueba el valor de un parametro segun lo que su ficha diga que tiene que ser.
pub fn check_value(name: &str, kind: Kind, value: &str) -> Result<(), String> {
    match kind {
        Kind::Text => Ok(()),
        Kind::Number => {
            let number: Result<i64, _> = value.parse();

            if number.is_err() || number.unwrap_or(0) <= 0 {
                return Err(format!(
                    "ACTION_PARAM: `{name}` tiene que ser un numero mayor que cero"
                ));
            }

            Ok(())
        }
        Kind::Ident => {
            let ok = !value.is_empty()
                && value.chars().count() <= MAX_IDENT
                && value
                    .chars()
                    .all(|one| one.is_ascii_alphanumeric() || one == '-' || one == '_');

            ok.then_some(()).ok_or_else(|| {
                format!("ACTION_PARAM: `{name}` tiene que ser un nombre de pagina")
            })
        }
        Kind::Color => {
            let raw = value.strip_prefix('#').unwrap_or(value);
            let ok = matches!(raw.len(), 3 | 6)
                && raw.chars().all(|one| one.is_ascii_hexdigit());

            ok.then_some(()).ok_or_else(|| {
                format!("ACTION_PARAM: `{name}` tiene que ser un color, como #fcd535")
            })
        }
    }
}

/// Un identificador de pagina no necesita mas.
const MAX_IDENT: usize = 40;

/// Los que sortea `random_free_bank_account()`.
const ACCOUNT_DIGITS: usize = 9;

/// Una almohadilla empieza un comentario.
pub fn strip_comment(line: &str) -> &str {
    line.split('#').next().unwrap_or("").trim()
}

/// Una IP en cuatro tramos, tal como la escribe un jugador.
pub fn parse_ip(text: &str) -> Option<i64> {
    let mut value: i64 = 0;
    let mut parts = 0;

    for part in text.split('.') {
        let octet: i64 = part.parse().ok()?;

        if !(0..=255).contains(&octet) {
            return None;
        }

        value = (value << 8) | octet;
        parts += 1;
    }

    (parts == 4).then_some(value)
}

/// Si la orden abre pagina, manda al entrar o se pinta como control.
pub fn effect(spec: &Spec) -> Effect {
    spec.effect
}

fn find_tag(html: &str, tag: &str) -> Option<usize> {
    html.to_lowercase().find(tag)
}

//! El lenguaje que puede llevar un sitio dentro de `<action>`.
//!
//! No es JavaScript ni se le parece: es una lista de ordenes, una por linea, que el
//! navegador del juego lee y convierte en controles suyos. Ese rodeo no es un capricho.
//!
//! Un sitio se pinta en un `iframe` con `sandbox` y **sin permiso para ejecutar guiones**:
//! aunque llegara uno escrito, el navegador no lo correria. Si la navegacion viviera dentro
//! del marco haria falta codigo ahi dentro, y no lo hay. Por eso el bloque **declara** lo
//! que el sitio ofrece y es el juego quien lo pinta y lo obedece, desde fuera.
//!
//! La etiqueta es `<action>` y no `<script>` a proposito. `<script>` sigue prohibido y no
//! hace falta tocar esa regla: asi no hay forma de confundir una orden del juego con codigo,
//! y si este HTML acabara pintado en cualquier otro sitio, un `<action>` no seria nada.
//!
//! ```text
//! <action>
//!   page: inicio | Portada
//!   go_to: 45.123.10.5 | Nuestra sede
//! </action>
//! <h1>Bienvenido</h1>
//!
//! <action>
//!   page: pagar | Pagar la cuota
//!   transfer: 349463068 | Pagar ahora | amount=2500 concept=Cuota mensual
//! </action>
//! <p>Su cuota vence hoy.</p>
//! ```
//!
//! Los **botones no van aqui**: van escritos en el propio HTML, para que el autor los
//! coloque donde quiera y los vista con su CSS. Se marcan con atributos y los revisa
//! [`markup`]:
//!
//! ```html
//! <button data-action="link" data-target="cuotas">Ver cuotas</button>
//! <a data-action="go_to" data-target="45.123.10.5">Nuestra sede</a>
//! ```
//!
//! El bloque `<action>` se queda con lo que no es un boton: abrir paginas y mandar a otra
//! maquina al entrar.
//!
//! **Que ordenes existen se decide en [`catalog`]**, y anadir una es anadir una entrada
//! ahi: el troceado, la validacion y los mensajes salen de la descripcion. Ni este fichero
//! ni [`parser`] ni [`markup`] saben cuantas hay.
//!
//! Una orden declara una intencion y nada mas. Ninguna mueve dinero por si sola: `transfer`
//! y `charge` dejan escrito un control, y quien pide el PIN y llama al banco es el
//! navegador. Un sitio que cobrara solo por pintarse no seria una estafa, seria un robo.

pub mod catalog;
mod markup;
mod parser;

use serde::Serialize;

use crate::error::ApiError;

use super::document::Element;
use catalog::Effect;
use parser::Chunk;

/// Cuantas ordenes admite un sitio. Un sitio con doscientas no es un sitio.
const MAX_COMMANDS: usize = 64;

/// Lo que la pagina ofrece hacer: un enlace, un pago, lo que traiga el catalogo.
#[derive(Debug, Clone, Serialize)]
pub struct SiteAction {
    /// El verbo, tal como lo entiende el navegador para saber que pintar.
    pub kind: String,
    pub target: String,
    pub label: String,

    /// Lo que acompane a la orden: `amount`, `concept`, lo que declare su ficha.
    #[serde(skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub params: std::collections::BTreeMap<String, String>,
}

/// Una pagina del sitio, ya recortada: el navegador pinta solo la que este abierta.
#[derive(Debug, Clone, Serialize)]
pub struct SitePage {
    pub id: String,
    pub title: String,
    pub html: String,
    pub actions: Vec<SiteAction>,
}

/// Lo que el navegador necesita para pintar un sitio con navegacion.
#[derive(Debug, Clone, Serialize, Default)]
pub struct Site {
    pub pages: Vec<SitePage>,

    /// A donde manda el sitio nada mas entrar, si es que manda a algun sitio.
    pub redirect: Option<String>,
}

impl Site {
    fn plain(html: &str) -> Self {
        Self {
            pages: vec![SitePage {
                id: "index".to_owned(),
                title: String::new(),
                html: html.to_owned(),
                actions: Vec::new(),
            }],
            redirect: None,
        }
    }

    /// Si no hay nada que el navegador tenga que pintar aparte del propio HTML.
    pub fn is_plain(&self) -> bool {
        self.redirect.is_none()
            && self.pages.len() <= 1
            && self.pages.first().is_none_or(|page| page.actions.is_empty())
    }
}

/// Lee un sitio y lo deja listo para pintar.
///
/// Nunca falla: un sitio con ordenes rotas se sirve igual, saltandose lo que no entiende.
/// Quien avisa de los errores es [`check`], al guardar, que es cuando hay alguien delante
/// para leerlos.
pub fn parse(html: &str) -> Site {
    let mut site = Site::default();
    let mut cover = String::new();

    for chunk in parser::split(html) {
        match chunk {
            Chunk::Html(text) => match site.pages.last_mut() {
                Some(page) => page.html.push_str(text),
                None => cover.push_str(text),
            },
            Chunk::Actions(body) => {
                for line in body.lines().filter_map(parser::read).flatten() {
                    apply(&mut site, line);
                }
            }
        }
    }

    if site.pages.is_empty() {
        let mut plain = Site::plain(cover.trim());
        plain.redirect = site.redirect;
        return plain;
    }

    // Lo que se escribio antes de abrir la primera pagina es cabecera de todas, no una
    // pagina suelta. Casi siempre es el `<style>`: dejarlo en una pagina propia haria que
    // las demas se pintaran sin estilos en cuanto el visitante cambiara de pestana.
    let cover = cover.trim();

    if !cover.is_empty() {
        for page in &mut site.pages {
            page.html = format!("{cover}{}", page.html);
        }
    }

    site
}

/// Comprueba las ordenes de un sitio y devuelve el motivo del primer fallo.
///
/// Se llama al guardar, junto al resto de reglas del sanitizador: una orden que no se
/// entiende es una orden que el jugador tiene que poder arreglar antes de publicarla.
pub fn check(html: &str, elements: &[Element]) -> Result<(), ApiError> {
    let mut total = 0;
    let mut open = 0;

    for chunk in parser::split(html) {
        let Chunk::Actions(body) = chunk else {
            continue;
        };

        open += 1;

        for line in body.lines() {
            let Some(read) = parser::read(line) else {
                continue;
            };

            total += 1;

            if total > MAX_COMMANDS {
                return Err(ApiError::BadRequest(format!(
                    "ACTION_TOO_LONG: un sitio admite {MAX_COMMANDS} ordenes como mucho"
                )));
            }

            read.map_err(ApiError::BadRequest)?;
        }
    }

    // `split` corta cuando encuentra un bloque sin cerrar, asi que el que quede a medias no
    // llega a contarse. Compararlo con lo que hay escrito es lo que lo delata.
    if open < html.to_lowercase().matches("<action").count() {
        return Err(ApiError::BadRequest(
            "ACTION_UNCLOSED: falta el `</action>` de algun bloque".into(),
        ));
    }

    // Y los botones escritos en el propio HTML, que son la otra forma de declarar una orden.
    markup::check(elements).map_err(ApiError::BadRequest)?;

    Ok(())
}

fn apply(site: &mut Site, line: parser::Line) {
    let action = SiteAction {
        kind: line.spec.verb.to_owned(),
        target: line.target,
        label: line.label,
        params: line.params,
    };

    match parser::effect(line.spec) {
        Effect::OpensPage => site.pages.push(SitePage {
            id: action.target,
            title: action.label,
            html: String::new(),
            actions: Vec::new(),
        }),
        Effect::OnEnter => site.redirect = Some(action.target),
        Effect::Control => match site.pages.last_mut() {
            Some(page) => page.actions.push(action),
            None => site.pages.push(SitePage {
                id: "index".to_owned(),
                title: String::new(),
                html: String::new(),
                actions: vec![action],
            }),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Los tests hablan de HTML, y `check` ya pide el arbol: se lo damos igual que hace
    /// `sanitize`, que es el unico que lo llama de verdad.
    fn check(html: &str) -> Result<(), ApiError> {
        super::check(html, &crate::internet::document::parse(html))
    }

    #[test]
    fn un_sitio_sin_ordenes_es_una_pagina_sola() {
        let site = parse("<h1>Hola</h1>");

        assert!(site.is_plain());
        assert_eq!(site.pages.len(), 1);
        assert_eq!(site.pages[0].html, "<h1>Hola</h1>");
    }

    #[test]
    fn parte_el_html_por_paginas() {
        let site = parse(
            "<action>page: inicio | Portada</action><h1>Uno</h1>\
             <action>page: precios | Tarifas</action><p>Dos</p>",
        );

        assert_eq!(site.pages.len(), 2);
        assert_eq!(site.pages[0].id, "inicio");
        assert_eq!(site.pages[0].title, "Portada");
        assert_eq!(site.pages[0].html, "<h1>Uno</h1>");
        assert_eq!(site.pages[1].id, "precios");
        assert_eq!(site.pages[1].html, "<p>Dos</p>");
    }

    #[test]
    fn lo_anterior_a_la_primera_pagina_es_cabecera_de_todas() {
        let site = parse(
            "<style>p{color:red}</style>\
             <action>page: una</action><p>Uno</p>\
             <action>page: dos</action><p>Dos</p>",
        );

        // El estilo viaja con las dos, o la segunda se pintaria en blanco y negro.
        assert_eq!(site.pages.len(), 2);
        assert!(site.pages[0].html.starts_with("<style>p{color:red}</style>"));
        assert!(site.pages[1].html.starts_with("<style>p{color:red}</style>"));
        assert!(site.pages[1].html.ends_with("<p>Dos</p>"));
    }

    #[test]
    fn recoge_acciones_de_varios_tipos() {
        let site = parse(
            "<action>\n\
             link: precios | Ver tarifas\n\
             go_to: 45.123.10.5 | Sede\n\
             </action><p>Hola</p>",
        );

        let actions = &site.pages[0].actions;
        assert_eq!(actions.len(), 2);
        assert_eq!(actions[0].kind, "link");
        assert_eq!(actions[0].target, "precios");
        assert_eq!(actions[1].kind, "go_to");
        assert_eq!(actions[1].label, "Sede");
    }

    #[test]
    fn la_etiqueta_es_opcional() {
        let site = parse("<action>go_to: 45.123.10.5</action><p>x</p>");

        assert_eq!(site.pages[0].actions[0].label, "45.123.10.5");
    }

    #[test]
    fn guarda_la_redireccion() {
        let site = parse("<action>redirect: 10.0.0.1</action>");

        assert_eq!(site.redirect.as_deref(), Some("10.0.0.1"));
    }

    #[test]
    fn una_orden_con_parametros() {
        let site = parse(
            "<action>transfer: 349463068 | Pagar | amount=2500 concept=Cuota</action><p>x</p>",
        );

        let action = &site.pages[0].actions[0];
        assert_eq!(action.kind, "transfer");
        assert_eq!(action.params.get("amount").map(String::as_str), Some("2500"));
        assert_eq!(action.params.get("concept").map(String::as_str), Some("Cuota"));
    }

    #[test]
    fn los_comentarios_no_son_ordenes() {
        let site = parse("<action># un comentario\npage: uno</action><p>x</p>");

        assert_eq!(site.pages.len(), 1);
        assert_eq!(site.pages[0].id, "uno");
    }

    #[test]
    fn acepta_lo_que_entiende() {
        assert!(check("<action>page: inicio | Portada</action>").is_ok());
        assert!(check("<action>go_to: 45.123.10.5</action>").is_ok());
        assert!(check("<action>transfer: 349463068 | Pagar | amount=10</action>").is_ok());
        assert!(check("<p>sin ordenes</p>").is_ok());
    }

    #[test]
    fn rechaza_lo_que_no_son_ordenes() {
        let error = check("<action>alert('hola')</action>").unwrap_err();

        assert!(format!("{error:?}").contains("ACTION"));
    }

    #[test]
    fn rechaza_una_orden_que_no_existe() {
        assert!(check("<action>fetch: 45.123.10.5</action>").is_err());
    }

    #[test]
    fn rechaza_una_ip_que_no_lo_es() {
        assert!(check("<action>go_to: no-soy-una-ip</action>").is_err());
        assert!(check("<action>go_to: 300.1.1.1</action>").is_err());
    }

    #[test]
    fn rechaza_un_nombre_de_pagina_con_cosas_raras() {
        assert!(check("<action>page: con espacios</action>").is_err());
    }

    #[test]
    fn rechaza_un_bloque_sin_cerrar() {
        assert!(check("<action>page: uno").is_err());
    }

    #[test]
    fn exige_los_parametros_obligatorios() {
        let error = check("<action>transfer: 349463068 | Pagar</action>").unwrap_err();

        assert!(format!("{error:?}").contains("amount"));
    }

    #[test]
    fn un_banco_pide_las_credenciales_en_su_propio_html() {
        let html = r#"<input class="mio" data-field="account">
                      <input class="mio" data-field="pin" type="password">
                      <button class="btn" data-action="bank_login" data-then="cuenta">Entrar</button>
                      <b data-bind="balance">0</b>
                      <ul data-list="movements"><li data-bind="concept"></li></ul>"#;

        assert!(check(html).is_ok());
    }

    #[test]
    fn rechaza_un_campo_que_el_juego_no_sabe_leer() {
        let error = check(r#"<input data-field="usuario">"#).unwrap_err();

        assert!(format!("{error:?}").contains("ACTION_FIELD"));
    }

    #[test]
    fn rechaza_un_hueco_que_nadie_va_a_rellenar() {
        assert!(check(r#"<b data-bind="saldo"></b>"#).is_err());
        assert!(check(r#"<ul data-list="pagos"></ul>"#).is_err());
    }

    #[test]
    fn el_destino_tras_una_operacion_es_una_pagina() {
        assert!(check(r#"<button data-action="bank_login" data-then="cuenta">x</button>"#).is_ok());
        assert!(check(r#"<button data-action="bank_login" data-then="con espacios">x</button>"#).is_err());
    }

    #[test]
    fn rechaza_un_parametro_que_no_existe() {
        assert!(check("<action>go_to: 1.2.3.4 | ir | amount=5</action>").is_err());
    }

    #[test]
    fn rechaza_un_importe_que_no_es_numero() {
        assert!(check("<action>transfer: 349463068 | x | amount=gratis</action>").is_err());
        assert!(check("<action>transfer: 349463068 | x | amount=-5</action>").is_err());
    }

    #[test]
    fn rechaza_una_cuenta_que_no_lo_es() {
        assert!(check("<action>transfer: 123 | x | amount=5</action>").is_err());
    }
}

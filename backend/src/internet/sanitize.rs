//! Que se puede escribir en la pagina de una maquina.
//!
//! El cliente pinta estos sitios en un marco aislado y **sin permiso para ejecutar nada**,
//! asi que aunque llegara un `<script>` no correria. Eso no basta: el visor es una defensa
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
//!
//! # Como se comprueba
//!
//! **Por lista blanca y sobre el arbol parseado**, no buscando texto sospechoso. La version
//! anterior preguntaba si el HTML contenia `"<script"`, `"src="` o `"url("`, y una lista de
//! lo prohibido siempre va por detras: `src = "x"` con espacios pasaba, y un parrafo que
//! mencionara `url(` se rechazaba sin cargar nada. Ahora se lee con el mismo motor que un
//! navegador ([`super::document`]) y lo que no esta permitido, no entra. Lo que se le
//! escape a la lista sera algo nuevo que nadie ha permitido todavia, no una forma distinta
//! de escribir lo de siempre.

use crate::error::ApiError;

use super::css;
use super::document::{self, Element};

/// Doce mil caracteres dan para una pagina con su hoja dentro y no para un vertedero.
pub const MAX_LENGTH: usize = 16_384;

/// Lo que puede haber en un sitio.
///
/// `style` esta a proposito: es el punto, cada sitio trae su aspecto. `action` tambien, y
/// lo que lleva dentro lo revisa `webscript`. `html`, `head` y `body` estan porque el
/// parser envuelve todo fragmento en ellos; escribirlos a mano no rompe nada.
///
/// **No estan `img` ni `video` ni `audio`**, y no por descuido: sin poder traer un fichero
/// de fuera, una imagen es una etiqueta vacia. Que falte la etiqueta lo explica mejor que
/// un `src` rechazado.
const ALLOWED_TAGS: &[&str] = &[
    "html", "head", "body", "style", "action", //
    "div", "span", "p", "section", "article", "header", "footer", "main", "nav", "aside", //
    "h1", "h2", "h3", "h4", "h5", "h6", "hr", "br", //
    "b", "strong", "i", "em", "u", "s", "small", "mark", "sub", "sup", //
    "code", "pre", "kbd", "samp", "abbr", "blockquote", "q", "cite", "time", //
    "ul", "ol", "li", "dl", "dt", "dd", //
    "table", "thead", "tbody", "tfoot", "tr", "th", "td", "caption", //
    "a", "button", "input", "label",
];

/// Atributos que valen en cualquier elemento. Los `data-*` van aparte, por prefijo: son
/// los que usa el juego para reconocer botones y huecos, y los revisa `webscript`.
const ALLOWED_GLOBAL: &[&str] = &["class", "id", "title", "lang", "dir", "hidden"];

/// Y los que solo valen en el suyo.
const ALLOWED_PER_TAG: &[(&str, &[&str])] = &[
    ("a", &["href"]),
    ("th", &["colspan", "rowspan", "scope"]),
    ("td", &["colspan", "rowspan"]),
    ("label", &["for"]),
    (
        "input",
        &[
            "type",
            "placeholder",
            "value",
            "name",
            "maxlength",
            "readonly",
            "disabled",
            "autocomplete",
            "inputmode",
        ],
    ),
];

/// Los `type` de `input` que tienen sentido en un sitio del juego. Fuera quedan `image`
/// —que carga un fichero— y `file`, que subiria uno.
const ALLOWED_INPUT_TYPES: &[&str] = &[
    "text", "password", "number", "search", "tel", "email", "checkbox", "hidden",
];

pub fn check(html: &str) -> Result<(), ApiError> {
    if html.len() > MAX_LENGTH {
        return Err(ApiError::BadRequest(format!(
            "TOO_LONG: la pagina ocupa {} caracteres y el maximo es {MAX_LENGTH}",
            html.len()
        )));
    }

    let elements = document::parse(html);

    for element in &elements {
        check_element(element)?;
    }

    // Lo ultimo, cuando ya se sabe que no hay nada peligroso suelto: que lo que haya dentro
    // de un `<action>` sean ordenes del juego y no otra cosa.
    super::webscript::check(html, &elements)?;

    Ok(())
}

fn check_element(element: &Element) -> Result<(), ApiError> {
    if !ALLOWED_TAGS.contains(&element.name.as_str()) {
        return Err(ApiError::BadRequest(format!(
            "TAG_NOT_ALLOWED: <{}> no se permite. Un sitio es texto, estructura y CSS",
            element.name
        )));
    }

    for (name, value) in &element.attributes {
        check_attribute(element, name, value)?;
    }

    if element.name == "style" {
        css::check(&element.text, "la hoja de estilo").map_err(ApiError::BadRequest)?;
    }

    Ok(())
}

fn check_attribute(element: &Element, name: &str, value: &str) -> Result<(), ApiError> {
    // Un manejador tiene su propio mensaje porque es el error que mas se comete de buena
    // fe: quien lo escribe cree que esta poniendo un boton, no un guion.
    if name.starts_with("on") && name.len() > 2 {
        return Err(ApiError::BadRequest(format!(
            "HANDLER_NOT_ALLOWED: {name} es un manejador de eventos, y eso es codigo"
        )));
    }

    // El estilo suelto vale, pero se lee igual que la hoja: nada que cargue de fuera.
    if name == "style" {
        return css::check(value, "un atributo `style`").map_err(ApiError::BadRequest);
    }

    // Los `data-*` son el vocabulario del juego. Que existan lo decide `webscript`; aqui
    // basta con saber que no son codigo.
    if name.starts_with("data-") {
        return Ok(());
    }

    if ALLOWED_GLOBAL.contains(&name) {
        return Ok(());
    }

    let allowed_here = ALLOWED_PER_TAG
        .iter()
        .find(|(tag, _)| *tag == element.name)
        .is_some_and(|(_, names)| names.contains(&name));

    if !allowed_here {
        return Err(ApiError::BadRequest(format!(
            "ATTR_NOT_ALLOWED: `{name}` no se permite en <{}>",
            element.name
        )));
    }

    if element.name == "a" && name == "href" {
        return check_href(value);
    }

    if element.name == "input" && name == "type" {
        let kind = value.trim().to_lowercase();

        if !ALLOWED_INPUT_TYPES.contains(&kind.as_str()) {
            return Err(ApiError::BadRequest(format!(
                "ATTR_NOT_ALLOWED: `type=\"{kind}\"` no se permite. Los que valen son: {}",
                ALLOWED_INPUT_TYPES.join(", ")
            )));
        }
    }

    Ok(())
}

/// Un enlace solo puede llevar a un sitio: a este.
///
/// Salir se hace con `data-action="go_to"`, que es una orden del juego y pasa por el
/// navegador. Un `href` a otra maquina de verdad sacaria al jugador del juego, y uno a
/// `http://` cargaria la pagina de un tercero dentro del marco, que es exactamente el
/// contador de visitas que esto viene a impedir.
fn check_href(value: &str) -> Result<(), ApiError> {
    let target = value.trim();

    if target.starts_with('#') {
        return Ok(());
    }

    Err(ApiError::BadRequest(format!(
        "LINK_NOT_ALLOWED: `href=\"{target}\"` sale del sitio. Dentro de la pagina vale \
         `href=\"#donde-sea\"`; para ir a otra maquina, `data-action=\"go_to\"`"
    )))
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

    //
    // Lo que el filtro por subcadenas dejaba pasar.
    //

    #[test]
    fn el_espacio_ya_no_esquiva_la_carga() {
        // `contains("src=")` no encontraba nada aqui, y el navegador carga la imagen igual.
        assert!(rejects("<img src = \"http://tracker.example/pixel.gif\">"));
        assert!(rejects("<style>body{background:url ('http://x.example/a.png')}</style>"));
    }

    #[test]
    fn lo_que_no_es_una_etiqueta_conocida_tampoco_entra() {
        // El parser no saca un `<script>` de aqui, saca un elemento con un nombre absurdo.
        // Con una lista de lo prohibido habria que haberlo previsto; con una de lo
        // permitido, no hace falta saber que era.
        assert!(rejects("<scr<script>ipt>alert(1)</script>"));
        assert!(rejects("<marquee>hola</marquee>"));
    }

    #[test]
    fn lo_que_carga_de_fuera_no_entra_aunque_cambie_de_nombre() {
        assert!(rejects("<svg><use href=\"http://x.example/a.svg#i\"></use></svg>"));
        assert!(rejects("<video><source src=\"http://x.example/a.mp4\"></video>"));
        assert!(rejects("<div style=\"background:url(http://x.example/a.png)\">x</div>"));
    }

    #[test]
    fn un_enlace_no_saca_del_sitio() {
        assert!(rejects("<a href=\"http://x.example\">fuera</a>"));
        assert!(check("<a href=\"#precios\">dentro</a>").is_ok());
        assert!(check("<a data-action=\"go_to\" data-target=\"45.123.10.5\">ir</a>").is_ok());
    }

    //
    // Lo que rechazaba de mas.
    //

    #[test]
    fn hablar_de_una_carga_no_es_hacerla() {
        assert!(check("<p>escribe url(...) en la hoja y no te dejara</p>").is_ok());
        assert!(check("<p>el atributo src= no se permite</p>").is_ok());
    }

    //
    // Que las paginas que ya existen siguen siendo validas.
    //

    #[test]
    fn la_plantilla_de_un_npc_sigue_valiendo() {
        let html = r#"<style>
            body{margin:0;background:#12100c;color:#e8dfc8;font:14px/1.6 Georgia,serif}
            header{padding:28px 32px;border-bottom:3px double #7a5c1f}
            h1{margin:0;font-size:26px;letter-spacing:.5px;color:#e8c46a}
            .sub{margin:6px 0 0;color:#8c7a55;font-style:italic}
            table{width:100%;border-collapse:collapse;font-size:13px}
            th{text-align:left;padding:6px 8px;border-bottom:2px solid #3a3120}
          </style>
          <header>
            <h1>Meridian Labs</h1>
            <p class="sub">Investigacion aplicada desde 1987</p>
          </header>
          <main>
            <p class="notice">Acceso al repositorio restringido al personal acreditado.</p>
            <table>
              <tr><th>Referencia</th><th>Linea de trabajo</th><th>Estado</th></tr>
              <tr><td class="ref">AX-114</td><td>Sintesis de materiales</td><td>en curso</td></tr>
            </table>
          </main>
          <footer>Meridian Labs &mdash; documento generado automaticamente</footer>"#;

        assert!(check(html).is_ok());
    }

    #[test]
    fn un_banco_con_sus_campos_y_sus_botones_sigue_valiendo() {
        let html = r#"<style>.caja{padding:1rem}</style>
          <div class="caja" data-when="out">
            <input data-field="account" type="text" placeholder="Cuenta">
            <input data-field="pin" type="password" placeholder="PIN">
            <button data-action="bank_login">Entrar</button>
          </div>
          <div class="caja" data-when="in">
            <p>Saldo: <span data-bind="balance">--</span></p>
            <ul data-list="movements">
              <li><span data-bind="concept"></span> <span data-bind="amount"></span></li>
            </ul>
            <button data-action="bank_logout">Salir</button>
          </div>"#;

        assert!(check(html).is_ok());
    }
}


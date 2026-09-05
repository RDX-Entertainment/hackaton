//! El HTML de un sitio, leido una sola vez.
//!
//! Antes cada comprobacion se buscaba la vida por su cuenta sobre el texto crudo: el
//! sanitizador con `contains("<script")` y `markup` retrocediendo hasta el `<` mas cercano.
//! Dos lecturas distintas del mismo HTML, y ninguna de las dos era la del navegador.
//!
//! Aqui se parsea con el mismo motor que usa un navegador (`html5ever`, via `scraper`) y lo
//! que sale es la lista de elementos que **de verdad** habra en la pagina. Eso cierra de
//! golpe toda una familia de trucos —`<scr<script>ipt>`, un comentario partiendo la
//! etiqueta, `src = "x"` con espacios— sin tener que preverlos uno a uno: el arbol no
//! depende de como se escribio, sino de como se lee.

use scraper::{Html, Node};

/// Un elemento de la pagina, con lo unico que hace falta para juzgarlo.
pub struct Element {
    /// En minusculas: el parser normaliza el nombre, asi que `<DIV>` y `<div>` son el mismo.
    pub name: String,

    /// Igual que el nombre, ya normalizados. `ONCLICK=` y `onclick=` llegan aqui iguales.
    pub attributes: Vec<(String, String)>,

    /// El texto directo que cuelga del elemento. Solo se mira en `<style>`, para pasarle la
    /// hoja al lector de CSS.
    pub text: String,
}

impl Element {
    pub fn attribute(&self, name: &str) -> Option<&str> {
        self.attributes
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.as_str())
    }
}

/// Todos los elementos de la pagina, en el orden en que aparecen.
pub fn parse(html: &str) -> Vec<Element> {
    let tree = Html::parse_fragment(html);

    tree.tree
        .nodes()
        .filter_map(|node| {
            let element = node.value().as_element()?;

            let text = node
                .children()
                .filter_map(|child| match child.value() {
                    Node::Text(chunk) => Some(chunk.to_string()),
                    _ => None,
                })
                .collect();

            Some(Element {
                name: element.name().to_owned(),
                attributes: element
                    .attrs()
                    .map(|(name, value)| (name.to_lowercase(), value.to_owned()))
                    .collect(),
                text,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(html: &str) -> Vec<String> {
        parse(html).into_iter().map(|element| element.name).collect()
    }

    #[test]
    fn normaliza_mayusculas() {
        assert!(names("<DIV ONCLICK='x'>hola</DIV>").contains(&"div".to_owned()));

        let elements = parse("<div ONCLICK='x'></div>");
        let div = elements.iter().find(|one| one.name == "div").unwrap();

        assert_eq!(div.attribute("onclick"), Some("x"));
    }

    #[test]
    fn ve_el_atributo_aunque_se_escriba_con_espacios() {
        // Esto es lo que se le escapaba a `contains("src=")`, y el navegador lo carga igual.
        let elements = parse("<img\n  src = \"http://x.example/p.gif\">");
        let img = elements.iter().find(|one| one.name == "img").unwrap();

        assert_eq!(img.attribute("src"), Some("http://x.example/p.gif"));
    }

    #[test]
    fn el_contenido_de_style_llega_entero() {
        let elements = parse("<style>body{color:red}</style>");
        let style = elements.iter().find(|one| one.name == "style").unwrap();

        assert_eq!(style.text.trim(), "body{color:red}");
    }
}

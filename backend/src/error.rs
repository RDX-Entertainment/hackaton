use actix_web::{HttpResponse, ResponseError, http::StatusCode};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("authentication required")]
    Unauthorized,
    #[error("{0}")]
    BadRequest(String),
    #[error("not found")]
    NotFound,
    #[error("internal server error")]
    Internal(#[from] sqlx::Error),
}

/// El codigo que lleva dentro un mensaje, cuando lo lleva.
///
/// El juego ya venia contestando `NO_CERTIFICATION` o `DISK_FULL: no cabe`. A la cara del
/// jugador eso es horrible —y es lo que se le enseñaba, tal cual, en un parrafo gris— pero
/// resulta ser justo lo que hace falta para hablar tres idiomas: **el servidor no puede
/// mandar la frase, porque no sabe en que lengua lee quien pregunta**. Manda el codigo, y la
/// frase la escribe el cliente.
///
/// Se saca del propio mensaje en vez de cambiar los 251 sitios que devuelven errores. Asi los
/// que ya traen codigo empiezan a hablar hoy, los que no lo traen siguen contestando
/// exactamente lo que contestaban, y cada mensaje que se convierta en codigo se traduce solo
/// el dia que se convierta. Una barrida de 251 sitios habria sido el mismo resultado con
/// todas las papeletas de romper algo por el camino.
///
/// La forma es estrecha a proposito: mayusculas, digitos y guion bajo, y o es el mensaje
/// entero o es lo que va antes de los dos puntos. `invalid blueprint` no es un codigo, y
/// tiene que seguir sin serlo.
fn code_of(message: &str) -> Option<&str> {
    let head = message.split(':').next()?.trim_end();
    let bien = head.len() >= 3
        && head.len() <= 40
        && head.starts_with(|c: char| c.is_ascii_uppercase())
        && head
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_');
    bien.then_some(head)
}

/// El cuerpo de un error.
///
/// `error` se queda donde estaba y diciendo lo mismo: es el respaldo para todo lo que
/// todavia no tiene traduccion, y quitarlo dejaria mudo al cliente ante cualquier codigo que
/// no conozca. `code` es lo nuevo, y es lo que se traduce.
#[derive(Serialize)]
struct ErrorBody<'a> {
    error: &'a str,
    code: &'a str,
}

impl ApiError {
    /// Cada error tiene codigo. Los tres fijos lo tienen porque son suyo; el de peticion
    /// incorrecta lo saca de su mensaje, y si no lleva ninguno vale el generico: el cliente
    /// sabra que no sabe traducirlo y enseñara el texto que vino.
    fn code(&self) -> &str {
        match self {
            Self::Unauthorized => "UNAUTHORIZED",
            Self::NotFound => "NOT_FOUND",
            Self::Internal(_) => "INTERNAL",
            Self::BadRequest(message) => code_of(message).unwrap_or("BAD_REQUEST"),
        }
    }
}

impl ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(ErrorBody {
            error: &self.to_string(),
            code: self.code(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn un_mensaje_que_es_solo_codigo_es_el_codigo() {
        assert_eq!(code_of("NO_CERTIFICATION"), Some("NO_CERTIFICATION"));
    }

    #[test]
    fn el_codigo_es_lo_que_va_antes_de_los_dos_puntos() {
        assert_eq!(
            code_of("DISK_FULL: no hay espacio suficiente"),
            Some("DISK_FULL")
        );
    }

    #[test]
    fn una_frase_normal_no_es_un_codigo() {
        assert_eq!(code_of("invalid blueprint"), None);
        assert_eq!(code_of("bank account required"), None);
        // Y tampoco lo es una frase que empieza con una palabra en mayusculas.
        assert_eq!(code_of("SQL error: algo"), None);
    }

    #[test]
    fn los_errores_fijos_traen_el_suyo() {
        assert_eq!(ApiError::NotFound.code(), "NOT_FOUND");
        assert_eq!(ApiError::Unauthorized.code(), "UNAUTHORIZED");
        assert_eq!(ApiError::BadRequest("sin codigo".into()).code(), "BAD_REQUEST");
    }
}

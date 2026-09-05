//! Cuantas veces seguidas se puede llamar a la puerta.
//!
//! bcrypt con coste 13 hace que **probar** una contrasena cueste, pero no impide
//! **seguir probando**. Sin un limite, el login y el registro aceptan intentos hasta que
//! alguno acierte, y la cuenta de administrador se llama `adm` en toda instalacion que
//! salga del ejemplo. La auditoria del 2026-08-25 lo anoto como SEC-04.
//!
//! El limite va **solo en `/auth/login` y `/auth/register`**. El resto de la API exige un
//! token, asi que quien llega ahi ya paso por aqui, y limitar el juego entero convertiria
//! una partida movida en un error.

use std::net::IpAddr;

use actix_governor::governor::clock::{Clock, DefaultClock, QuantaInstant};
use actix_governor::governor::middleware::NoOpMiddleware;
use actix_governor::governor::NotUntil;
use actix_governor::{
    GovernorConfig, GovernorConfigBuilder, KeyExtractor, SimpleKeyExtractionError,
};
use actix_web::dev::ServiceRequest;
use actix_web::http::{header::ContentType, StatusCode};
use actix_web::{HttpResponse, HttpResponseBuilder};

/// Intentos permitidos de golpe: teclear mal la contrasena tres veces seguidas es humano.
const BURST: u32 = 8;

/// Y despues, uno cada tantos segundos. A ese ritmo un diccionario no termina nunca, y
/// quien de verdad se equivoco no llega a notarlo.
const SECONDS_PER_REQUEST: u64 = 15;

/// De donde sale la IP del que llama.
///
/// No vale `peer_addr`: en produccion la API solo la alcanza nginx desde la red interna de
/// Compose, asi que todas las peticiones parecerian venir de la misma maquina y el limite
/// seria uno global para todo el mundo. La IP de verdad llega en `X-Real-IP`, que
/// `deploy/nginx.conf` rellena a partir de `CF-Connecting-IP`.
///
/// **Fiarse de una cabecera solo es correcto porque nadie puede ponerla:** el unico camino
/// hasta la API es nginx, y el unico camino hasta nginx es el tunel de Cloudflare, que
/// reescribe `CF-Connecting-IP` con la IP real del visitante. Si algun dia el 8080 quedara
/// expuesto, esto pasaria a ser falsificable y habria que volver a `peer_addr`.
#[derive(Clone)]
pub struct ClientIp;

impl KeyExtractor for ClientIp {
    type Key = IpAddr;
    type KeyExtractionError = SimpleKeyExtractionError<&'static str>;

    fn extract(&self, req: &ServiceRequest) -> Result<Self::Key, Self::KeyExtractionError> {
        let forwarded = req
            .headers()
            .get("x-real-ip")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.trim().parse::<IpAddr>().ok());

        let direct = req.peer_addr().map(|socket| socket.ip());

        let ip = forwarded.or(direct).ok_or_else(|| {
            SimpleKeyExtractionError::new("no se pudo determinar el origen de la peticion")
        })?;

        // A un cliente IPv6 suele darsele un prefijo entero, asi que cambiar de direccion
        // dentro del suyo no cuesta nada. Se cuenta por prefijo /56, como hace el extractor
        // por defecto del crate.
        Ok(match ip {
            IpAddr::V6(address) => {
                let mut octets = address.octets();
                octets[7..16].fill(0);
                IpAddr::V6(octets.into())
            }
            other => other,
        })
    }

    /// El juego espera `{"error": "..."}` en cualquier fallo —lo lee `errorFrom` en
    /// `frontend/src/utils/api/client.ts`—, asi que el 429 tambien lo devuelve asi y la
    /// pantalla de entrada lo enseña sin tocar nada.
    fn exceed_rate_limit_response(
        &self,
        negative: &NotUntil<QuantaInstant>,
        mut response: HttpResponseBuilder,
    ) -> HttpResponse {
        let wait = negative.wait_time_from(DefaultClock::default().now()).as_secs();

        response
            .status(StatusCode::TOO_MANY_REQUESTS)
            .content_type(ContentType::json())
            .body(format!(
                r#"{{"error":"Demasiados intentos. Vuelve a probar en {wait} s"}}"#
            ))
    }
}

/// La configuracion que comparten las dos rutas de entrada.
///
/// Se construye una sola vez y se clona en cada worker: el estado va dentro de un `Arc`, asi
/// que los contadores son los mismos para todos. Construirla dentro del cierre de
/// `HttpServer::new` daria un cubo por hilo y multiplicaria el limite por el numero de
/// workers sin que se notara.
pub fn config() -> GovernorConfig<ClientIp, NoOpMiddleware> {
    GovernorConfigBuilder::default()
        .seconds_per_request(SECONDS_PER_REQUEST)
        .burst_size(BURST)
        .key_extractor(ClientIp)
        .finish()
        .expect("la configuracion del limite de intentos es constante y valida")
}

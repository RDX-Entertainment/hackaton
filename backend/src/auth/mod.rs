pub mod dtos;
pub mod handlers;
pub mod middleware;
pub mod models;
pub mod ratelimit;
pub mod repo;
pub mod services;

// Todo el crate pide `crate::auth::Ctx` en la firma de sus handlers, asi que el extractor
// se reexporta aqui y los demas modulos no tienen que saber donde vive.
pub use middleware::{AdminOnly, Ctx};

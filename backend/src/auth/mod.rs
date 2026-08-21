pub mod dtos;
pub mod handlers;
pub mod models;
pub mod repo;
pub mod services;

// Todo el crate pide `crate::auth::CurrentUser` en la firma de sus handlers, asi que el
// extractor se reexporta aqui y los demas modulos no tienen que saber que vive en models.
pub use models::CurrentUser;

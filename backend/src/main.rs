mod api;
mod auth;
mod error;
mod legacy;
mod state;

use actix_cors::Cors;
use actix_web::{App, HttpServer, middleware::Logger, web};
use dotenvy::dotenv;
use sqlx::mysql::MySqlPoolOptions;
use std::env;
use tracing_actix_web::TracingLogger;

use crate::state::AppState;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_owned());
    let port = env::var("PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(8080);
    let pool = MySqlPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .expect("unable to connect to MySQL");
    let state = web::Data::new(AppState { pool, jwt_secret });

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .wrap(TracingLogger::default())
            .wrap(Logger::default())
            .wrap(Cors::permissive())
            .configure(api::configure)
    })
    .bind((host, port))?
    .run()
    .await
}

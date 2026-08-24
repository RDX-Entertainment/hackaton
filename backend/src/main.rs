mod auth;
mod error;
mod hardware;
mod internet;
mod kits;
mod legacy;
mod mails;
mod finances;
mod forum;
mod missions;
mod news;
mod process;
mod social;
mod pages;
mod software;
mod state;
mod user;
mod wiki;

#[allow(non_snake_case)]
mod Generic;


use actix_cors::Cors;
use actix_web::{middleware::Logger, web, App, HttpServer};
use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::env;
use tracing_actix_web::TracingLogger;

use crate::state::AppState;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    env_logger::init_from_env(
        env_logger::Env::new()
            .default_filter_or("info"),
    );

    let database_url =
        env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set");

    let jwt_secret =
        env::var("JWT_SECRET")
            .expect("JWT_SECRET must be set");

    let host =
        env::var("HOST")
            .unwrap_or_else(|_| "127.0.0.1".to_owned());

    let port = env::var("PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(8080);

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .expect("unable to connect to PostgreSQL");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("unable to apply PostgreSQL migrations");

    if let Err(error) =
        auth::services::ensure_admin(&pool, &jwt_secret).await
    {
        return Err(std::io::Error::other(format!(
            "no se pudo preparar la cuenta de administrador: {error}"
        )));
    }

    let state = web::Data::new(AppState {
        pool,
        jwt_secret,
    });

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .wrap(TracingLogger::default())
            .wrap(Logger::default())
            .wrap(Cors::permissive())
            .service(
                web::scope("/api")
                    .configure(auth::handlers::configure)
                    .configure(user::handlers::configure)
                    .configure(wiki::handlers::configure)
                    .configure(software::handlers::configure)
                    .configure(internet::handlers::configure)
                    .configure(news::handlers::configure)
                    .configure(hardware::handlers::configure)
                    .configure(kits::handlers::configure)
                    .configure(mails::handlers::configure)
                    .configure(missions::handlers::configure)
                    .configure(forum::handlers::configure)
                    .configure(process::handlers::configure)
                    .configure(finances::handlers::configure)
                    .configure(social::handlers::configure)
                    .configure(pages::handlers::configure)
                    .service(
                        web::scope("/admin")
                            .wrap(auth::AdminOnly)
                            .configure(hardware::handlers::configure_admin)
                            .configure(kits::handlers::configure_admin)
                            .configure(internet::handlers::configure_admin)
                            .configure(user::handlers::configure_admin),
                    ),
            )
    })
    .bind((host, port))?
    .run()
    .await
}
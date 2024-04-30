mod constraint;
mod controller;
mod middleware;
mod solver;

use std::{env, io::Result};

use actix_web::{middleware::Logger, App, HttpServer};
use dotenv::dotenv;
use env_logger::Env;
use log::info;

#[actix_web::main]
async fn main() -> Result<()> {
    dotenv().ok();
    env_logger::init_from_env(Env::default().default_filter_or("info")); // RUST_LOG

    let mode = env::var("MODE").unwrap_or("testing".to_owned());
    let host = if mode == "prod" {
        "0.0.0.0"
    } else {
        "127.0.0.1"
    };
    let port = env::var("PORT")
        .unwrap_or("8080".to_owned())
        .parse::<u16>()
        .expect("Failed to parse the host port number");

    info!("Starting a listener on {}:{}", host, port);

    // Only panics if no socket addresses were successfully bound or if no Tokio runtime is set up
    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .service(controller::solve)
    })
    .bind((host, port))?
    .run()
    .await
}

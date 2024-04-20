mod controller;
mod middleware;
mod solve;

use std::{env, io::Result};

use actix_web::{App, HttpServer};
use dotenv::dotenv;

#[actix_web::main]
async fn main() -> Result<()> {
    dotenv().ok();

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

    println!("Starting a listener on {}:{}", host, port);

    HttpServer::new(|| App::new().service(controller::solve))
        .bind((host, port))?
        .run()
        .await
}

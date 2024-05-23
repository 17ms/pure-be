mod controller;
mod dfs;
mod dlx;
mod solver;
mod sudoku;

use std::{env, io::Result};

use actix_governor::{
    governor::middleware::StateInformationMiddleware, Governor, GovernorConfig,
    GovernorConfigBuilder, PeerIpKeyExtractor,
};
use actix_web::{middleware::Logger, App, HttpServer};
use dotenv::dotenv;
use env_logger::Env;
use log::info;

#[derive(Debug)]
struct Conf {
    host: String,
    port: u16,
    governor_conf: GovernorConfig<PeerIpKeyExtractor, StateInformationMiddleware>,
}

impl Conf {
    fn new() -> Self {
        dotenv().ok();

        // Logging
        env_logger::init_from_env(Env::default().default_filter_or("info"));

        // Socket bindings
        let host = match env::var("MODE").unwrap_or("testing".into()).as_str() {
            "prod" => String::from("0.0.0.0"),
            _ => String::from("127.0.0.1"),
        };
        let port = env::var("PORT")
            .unwrap_or("8080".into())
            .parse::<u16>()
            .expect("Failed to parse the host port number");

        // Rate limiting
        let interval_s = env::var("QUOTA_REPLENISH_INTERVAL_S")
            .unwrap_or("60".into())
            .parse::<u64>()
            .expect("Failed to parse the quota replenish interval");
        let burst_size = env::var("BURST_SIZE")
            .unwrap_or("3".into())
            .parse::<u32>()
            .expect("Failed to parse the burst size");
        let governor_conf = GovernorConfigBuilder::default()
            .per_second(interval_s)
            .burst_size(burst_size)
            .use_headers()
            .finish()
            .expect("Failed to generate a config for the rate limiter");

        Self {
            host,
            port,
            governor_conf,
        }
    }
}

#[actix_web::main]
async fn main() -> Result<()> {
    let conf = Conf::new();

    info!("Starting a listener on {}:{}", conf.host, conf.port);

    // Only panics if no socket addresses were successfully bound or if no Tokio runtime is set up
    HttpServer::new(move || {
        App::new()
            .wrap(Governor::new(&conf.governor_conf))
            .wrap(Logger::default())
            .service(controller::solve)
    })
    .bind((conf.host, conf.port))?
    .run()
    .await
}

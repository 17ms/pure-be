use actix_web::{post, web, HttpResponse, Responder};
use log::info;
use serde::{Deserialize, Serialize};

use crate::solver::{handle_req, Sudoku};

#[derive(Deserialize)]
struct Entry {
    grid: String,
}

#[derive(Serialize)]
struct Response {
    data: Vec<Sudoku>,
    total_cpu_ms: u128,
}

#[post("/solve")]
pub async fn solve(entries: web::Json<Vec<Entry>>) -> impl Responder {
    let mut data = Vec::new();

    for e in entries.iter() {
        data.push(Sudoku::new(e.grid.to_owned()));
    }

    // solution, cpu time (ms), branch count, visited nodes count
    let total_cpu_ms =
        handle_req(&mut data).expect("Error during request handling on route '/solve'");
    let res = Response { data, total_cpu_ms };

    info!("Processed {} entries in {} ms", entries.len(), total_cpu_ms);

    HttpResponse::Ok().json(res)
}

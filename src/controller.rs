use actix_web::{post, web, HttpResponse, Responder};
use serde::Deserialize;

use crate::solve::{handle_req, Sudoku};

#[derive(Deserialize)]
struct Entry {
    grid: String,
}

#[post("/solve")]
pub async fn solve(entries: web::Json<Vec<Entry>>) -> impl Responder {
    let mut data = Vec::new();

    for e in entries.iter() {
        data.push(Sudoku::new(e.grid.to_owned()));
    }

    // solution, cpu time (ms), branch count, visited nodes count
    let _total_cpu_ms =
        handle_req(&mut data).expect("Error during request handling on route '/solve'");

    // TODO: include total cpu time into the response

    HttpResponse::Ok().json(data)
}

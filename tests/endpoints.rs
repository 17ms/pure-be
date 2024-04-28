use std::{
    fs::File,
    io::{BufRead, BufReader},
};

use actix_web::{test, App};
use pure_be::{controller, solver::has_unique_items};
use rand::Rng;

const COLLECTION_SIZE: usize = 49150;

/// Sends a POST request with randomly selected sudokus to the '/sdfs' endpoint.
#[actix_web::test]
async fn test_sdfs() {
    let app = test::init_service(App::new().service(controller::solve)).await;
    let unsolved = get_unsolved();
    let payload = create_payload(unsolved);

    let req = test::TestRequest::post()
        .uri("/sdfs")
        .set_json(payload)
        .to_request();
    let res: controller::Response = test::call_and_read_body_json(&app, req).await;

    for sudoku in res.data {
        assert_constraints(&sudoku.grid);
    }
}

fn get_unsolved() -> Vec<String> {
    let mut rng = rand::thread_rng();

    let file = File::open("./tests/sudoku17")
        .expect("Failed to open the 'sudoku17' collection file for reading");
    let lines: Vec<String> = BufReader::new(file).lines().map_while(Result::ok).collect();
    let mut unsolved = Vec::new();

    for _ in 0..3 {
        let ln = rng.gen_range(0..COLLECTION_SIZE);
        unsolved.push(lines[ln].to_owned());
    }

    unsolved
}

fn create_payload(raws: Vec<String>) -> Vec<controller::Entry> {
    raws.iter()
        .map(|raw| controller::Entry {
            grid: raw.to_owned(),
        })
        .collect()
}

fn assert_constraints(grid: &[Vec<u8>]) {
    // No size constraints need to be checked due to other than
    // 9x9 Sudokus being quite rare in the wild.

    let square_count = grid.len() / 3;

    for row in grid {
        assert!(has_unique_items(row), "Row constraint not fulfilled");
    }

    for col_idx in 0..grid.len() {
        let col = grid.iter().map(|row| row[col_idx]);
        assert!(has_unique_items(col), "Column constraint not fulfilled");
    }

    for br in 0..square_count {
        for bc in 0..square_count {
            let boxy = grid
                .iter()
                .skip(br * 3)
                .take(3)
                .flat_map(|row| row.iter().skip(bc * 3).take(3));
            assert!(
                has_unique_items(boxy),
                "3x3 square constraint not fulfilled"
            );
        }
    }
}

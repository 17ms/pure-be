use std::{
    fs::File,
    io::{BufRead, BufReader},
};

use actix_web::{http::StatusCode, test, App};
use pure_be::{
    constraint::check_default_constraints,
    controller::{solve, Entry, ErrorResponse, Response},
};
use rand::Rng;

const COLLECTION_SIZE: usize = 49150;

/// Sends a POST request with randomly picked Sudokus to the '/sdfs' endpoint to test the solving process.
#[actix_web::test]
async fn test_sdfs() {
    let app = test::init_service(App::new().service(solve)).await;
    let unsolved = get_unsolved();
    let payload = create_payload(unsolved);

    let req = test::TestRequest::post()
        .uri("/sdfs")
        .set_json(payload)
        .to_request();
    let res: Response = test::call_and_read_body_json(&app, req).await;

    for sudoku in res.data {
        assert!(check_default_constraints(&sudoku.grid, None).unwrap());
    }
}

/// Sends a POST request with syntactically malformed contents to test the regex validators.
#[actix_web::test]
async fn test_malformed_data() {
    let app = test::init_service(App::new().service(solve)).await;

    let total_raws = vec![
        "00080905160020000C30000000001000003008A90000000000040040003060B000051000000000000", // Invalid contents
        "0008051600200000300000000010000030080900000000000400400030600000051000000000", // Invalid length
    ];

    for raw in total_raws {
        let payload = create_payload(vec![raw.to_owned()]);
        let req = test::TestRequest::post()
            .uri("/sdfs")
            .set_json(payload)
            .to_request();
        let res = test::call_service(&app, req).await;

        assert_eq!(
            res.status(),
            StatusCode::BAD_REQUEST,
            "Malformed data should result in a 400 (Bad Request) response"
        );

        let res_body: ErrorResponse = test::read_body_json(res).await;

        assert_eq!(
            res_body.code,
            StatusCode::BAD_REQUEST
                .to_string()
                .split(' ')
                .collect::<Vec<&str>>()[0],
            "Invalid status code received in the error payload"
        );
        assert_eq!(
            res_body.message,
            "The entries must be syntactically valid and fulfill the basic Sudoku constraints",
            "Invalid message received in the error payload"
        )
    }
}

/// Sends a POST request with invalid Sudoku grid (i.e. puzzle constraints are not fulfilled).
#[actix_web::test]
async fn test_invalid_sudoku() {
    let app = test::init_service(App::new().service(solve)).await;

    let invalid_raw =
        "830070000600195000098000060800060003400803001700020006060000280000419005000080079";
    let payload = create_payload(vec![invalid_raw.to_owned()]);

    let req = test::TestRequest::post()
        .uri("/sdfs")
        .set_json(payload)
        .to_request();
    let res = test::call_service(&app, req).await;

    assert_eq!(
        res.status(),
        StatusCode::BAD_REQUEST,
        "Invalid HTTP status code received in the error response"
    );

    let res_body: ErrorResponse = test::read_body_json(res).await;

    assert_eq!(
        res_body.code,
        StatusCode::BAD_REQUEST
            .to_string()
            .split(' ')
            .collect::<Vec<&str>>()[0],
        "Invalid status code received in the error payload"
    );
    assert_eq!(
        res_body.message,
        "The entries must be syntactically valid and fulfill the basic Sudoku constraints",
        "Invalid message received in the error payload"
    )
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

fn create_payload(raws: Vec<String>) -> Vec<Entry> {
    raws.iter()
        .map(|raw| Entry {
            grid: raw.to_owned(),
        })
        .collect()
}

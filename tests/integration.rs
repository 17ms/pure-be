use std::{
    fs::File,
    io::{BufRead, BufReader},
};

use actix_web::{http::StatusCode, test, App};
use pure_be::{
    controller::{self, Entry, ErrorResponse, SuccessResponse},
    sudoku::Sudoku,
};
use rand::Rng;

/// Sends a POST request with randomly picked Sudokus to the `/solve` endpoint with the
/// `solver_type` parameter set to `dfs` to test the AC-3 + enhanced DFS implementation.
#[actix_web::test]
async fn test_dfs_solver() {
    let test_app = test::init_service(App::new().service(controller::solve)).await;
    let unsolved = get_unsolved();
    let payload = into_payload(unsolved, Some(String::from("dfs")));

    let req = test::TestRequest::post()
        .uri("/solve")
        .set_json(payload)
        .to_request();
    let res: SuccessResponse = test::call_and_read_body_json(&test_app, req).await;

    for grid_str in res.get_solved() {
        let sudoku = Sudoku::new(grid_str).unwrap();
        assert!(sudoku.is_valid(None));
        assert!(sudoku.is_solved());
    }
}

/// Sends a POST request with randomly picked Sudokus to the `/solve` endpoint with the
/// `solver_type` parameter set to `dlx` to test the Algorithm X (exact cover) implementation.
#[actix_web::test]
async fn test_dlx_solver() {
    let test_app = test::init_service(App::new().service(controller::solve)).await;
    let unsolved = get_unsolved();
    let payload = into_payload(unsolved, Some(String::from("dlx")));

    let req = test::TestRequest::post()
        .uri("/solve")
        .set_json(payload)
        .to_request();
    let res: SuccessResponse = test::call_and_read_body_json(&test_app, req).await;

    for grid_str in res.get_solved() {
        let sudoku = Sudoku::new(grid_str).unwrap();
        assert!(sudoku.is_valid(None));
        assert!(sudoku.is_solved());
    }
}

/// Sends a POST request with syntactically malformed contents to test the regex validators.
#[actix_web::test]
async fn test_malformed_data() {
    let test_app = test::init_service(App::new().service(controller::solve)).await;

    let total_raws = vec![
        "00080905160020000C30000000001000003008A90000000000040040003060B000051000000000000", // Invalid contents
        "0008051600200000300000000010000030080900000000000400400030600000051000000000", // Invalid length
    ];

    for raw in total_raws {
        let payload = into_payload(vec![raw.to_owned()], None);
        let req = test::TestRequest::post()
            .uri("/solve")
            .set_json(payload)
            .to_request();
        let res = test::call_service(&test_app, req).await;

        assert_eq!(
            res.status(),
            StatusCode::BAD_REQUEST,
            "Invalid HTTP status code received in the error response"
        );

        let res_body: ErrorResponse = test::read_body_json(res).await;
        let e_status = res_body.status().unwrap();

        assert_eq!(
            e_status,
            StatusCode::BAD_REQUEST,
            "Invalid HTTP status code received in the error payload"
        );
    }
}

/// Sends a POST request with invalid Sudoku grid (i.e. puzzle constraints are not fulfilled) to
/// test the `Entry` to `Sudoku` conversion process via the `to_sudoku` method.
#[actix_web::test]
async fn test_invalid_grid() {
    let test_app = test::init_service(App::new().service(controller::solve)).await;

    let invalid_raw =
        "830070000600195000098000060800060003400803001700020006060000280000419005000080079";
    let payload = into_payload(vec![invalid_raw.to_owned()], None);

    let req = test::TestRequest::post()
        .uri("/solve")
        .set_json(payload)
        .to_request();
    let res = test::call_service(&test_app, req).await;

    assert_eq!(
        res.status(),
        StatusCode::BAD_REQUEST,
        "Invalid HTTP status code received in the error response"
    );

    let res_body: ErrorResponse = test::read_body_json(res).await;
    let e_status = res_body.status().unwrap();

    assert_eq!(
        e_status,
        StatusCode::BAD_REQUEST,
        "Invalid HTTP status code received in the error payload"
    );
}

fn get_unsolved() -> Vec<String> {
    static COLLECTION_SIZE: usize = 49150;
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

fn into_payload(raws: Vec<String>, s_type: Option<String>) -> Vec<Entry> {
    raws.into_iter()
        .map(|raw| Entry::new(raw.to_owned(), s_type.clone()))
        .collect()
}

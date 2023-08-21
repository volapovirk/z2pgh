use std::net::TcpListener;

use async_std::task;
use surf::StatusCode;

fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let _ = task::spawn(z2pgh::run(listener));

    format!("http://127.0.0.1:{port}")
}

#[async_std::test]
async fn health_check_works() {
    let address = spawn_app();

    let client = surf::Client::new();
    let request = surf::get(format!("{address}/health_check"));
    let response = client
        .send(request)
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    assert!(response.is_empty().is_none());
}

#[async_std::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    let address = spawn_app();

    let client = surf::Client::new();
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let request = surf::post(format!("{address}/subscriptions"))
        .body(body)
        .content_type("application/x-www-form-urlencoded");
    let response = client
        .send(request)
        .await
        .expect("Failed to execute request.");

    assert_eq!(StatusCode::Ok, response.status());
}

#[async_std::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    let address = spawn_app();
    let client = surf::Client::new();
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];
    for (invalid_body, error_message) in test_cases {
        let request = surf::post(format!("{address}/subscriptions"))
            .body(invalid_body)
            .content_type("application/x-www-form-urlencoded");

        let response = client
            .send(request)
            .await
            .expect("Failed to execute request.");

        assert_eq!(
            StatusCode::UnprocessableEntity,
            response.status(),
            "The API did not fail with 422 Unprocessable Content when the payload was {}.",
            error_message
        );
    }
}

use std::net::TcpListener;

use async_std::task;

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

fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let _ = task::spawn(z2pgh::run(listener));

    format!("http://127.0.0.1:{port}")
}

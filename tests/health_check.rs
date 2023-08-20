use async_std::task;

#[async_std::test]
async fn health_check_works() {
    spawn_app();

    let client = surf::Client::new();
    let request = surf::get("http://127.0.0.1:8000/health_check");
    let response = client
        .send(request)
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    assert!(response.is_empty().is_none());
}

fn spawn_app() {
    let _ = task::spawn(z2pgh::run("127.0.0.1:8000"));
}

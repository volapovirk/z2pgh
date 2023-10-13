use crate::helpers::spawn_app;

#[async_std::test]
async fn health_check_works() {
    let app = spawn_app().await;

    let client = surf::Client::new();
    let request = surf::get(format!("{}/health_check", app.address));
    let response = client
        .send(request)
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    assert!(response.is_empty().is_none());
}

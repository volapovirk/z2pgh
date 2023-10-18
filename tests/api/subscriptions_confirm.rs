use surf::{Client, StatusCode, Url};
use wiremock::{
    matchers::{method, path},
    Mock, ResponseTemplate,
};

use crate::helpers::spawn_app;

#[async_std::test]
async fn confirmations_without_token_are_rejected_with_a_400() {
    let app = spawn_app().await;
    let client = Client::new();

    let request = surf::get(&format!("{}/subscriptions/confirm", app.address));
    let response = client
        .send(request)
        .await
        .expect("Failed to execute request.");

    assert_eq!(StatusCode::BadRequest, response.status());
}

#[async_std::test]
async fn the_link_returned_by_subsribe_returns_a_200_if_called() {
    let app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    app.post_subscriptions(body.into()).await;

    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    let confirmation_links = app.get_confirmation_links(&email_request);

    let client = Client::new();
    let request = surf::get(confirmation_links.html);
    let response = client
        .send(request)
        .await
        .expect("Failed to execute request.");

    assert_eq!(StatusCode::Ok, response.status());
}

#[async_std::test]
async fn clicking_on_the_confirmation_link_confirms_a_subscriber() {
    let app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    app.post_subscriptions(body.into()).await;

    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    let confirmation_links = app.get_confirmation_links(&email_request);

    let client = Client::new();
    let request = surf::get(confirmation_links.html);
    client
        .send(request)
        .await
        .expect("Failed to execute request.");

    let saved = sqlx::query!("SELECT email, name, status FROM subscriptions",)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
    assert_eq!(saved.status, "confirmed");
}

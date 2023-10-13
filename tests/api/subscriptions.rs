use surf::StatusCode;

use crate::helpers::spawn_app;

#[async_std::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    let app = spawn_app().await;

    let client = surf::Client::new();
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let request = surf::post(format!("{}/subscriptions", app.address))
        .body(body)
        .content_type("application/x-www-form-urlencoded");
    let response = client
        .send(request)
        .await
        .expect("Failed to execute request.");

    assert_eq!(StatusCode::Ok, response.status());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
}

#[async_std::test]
async fn subscribe_returns_a_422_when_data_is_missing() {
    let app = spawn_app().await;
    let client = surf::Client::new();
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];
    for (invalid_body, error_message) in test_cases {
        let request = surf::post(format!("{}/subscriptions", app.address))
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

#[async_std::test]
async fn subscribe_returns_a_400_when_fields_are_present_but_empty() {
    let app = spawn_app().await;
    let client = surf::Client::new();
    let test_cases = vec![
        ("name=&email=usula_le_guin%40gmail.com", "empty name"),
        ("name=Ursula&email=", "empty email"),
        ("name=Ursula&email=definitely-not-an-email", "invalid email"),
    ];
    for (body, description) in test_cases {
        let request = surf::post(format!("{}/subscriptions", app.address))
            .body(body)
            .content_type("application/x-www-form-urlencoded");

        let response = client
            .send(request)
            .await
            .expect("Failed to execute request.");

        assert_eq!(
            StatusCode::BadRequest,
            response.status(),
            "The API did not return a 400 Bad Requestwhen the payload was {}.",
            description
        );
    }
}

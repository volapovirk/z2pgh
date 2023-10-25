use crate::helpers::{spawn_app, ConfirmationLinks, TestApp};
use surf::{Client, StatusCode};
use wiremock::matchers::{any, method, path};
use wiremock::{Mock, ResponseTemplate};

#[async_std::test]
async fn newsletters_are_not_delivered_to_unconfirmed_subscribers() {
    // Arrange
    let app = spawn_app().await;
    create_unconfirmed_subscriber(&app).await;
    Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        // We assert that no request is fired at Postmark!
        .expect(0)
        .mount(&app.email_server)
        .await;
    // Act
    // A sketch of the newsletter payload structure.
    // We might change it later on.
    let newsletter_request_body = serde_json::json!({
        "title": "Newsletter title",
        "content": {
            "text": "Newsletter body as plain text",
            "html": "<p>Newsletter body as HTML</p>",
        }
    });
    let response = app.post_newsletters(newsletter_request_body).await;

    // Assert
    assert_eq!(response.status(), StatusCode::Ok);
    // Mock verifies on Drop that we haven't sent the newsletter email
}

#[async_std::test]
async fn newsletters_are_delivered_to_confirmed_subscribers() {
    // Arrange
    let app = spawn_app().await;
    create_confirmed_subscriber(&app).await;
    Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;
    // Act
    // A sketch of the newsletter payload structure.
    // We might change it later on.
    let newsletter_request_body = serde_json::json!({
        "title": "Newsletter title",
        "content": {
            "text": "Newsletter body as plain text",
            "html": "<p>Newsletter body as HTML</p>",
        }
    });

    let response = app.post_newsletters(newsletter_request_body).await;

    // Assert
    assert_eq!(response.status(), StatusCode::Ok);
    // Mock verifies on Drop that we haven't sent the newsletter email
}

#[async_std::test]
async fn newsletters_returns_422_for_invalid_data() {
    let app = spawn_app().await;
    let test_cases = vec![
        (
            serde_json::json!({
                "content": {
                    "text": "Newsletter body as plain text",
                    "html": "<p>Newsletter body as HTML</p>",
                }
            }),
            "missing title",
        ),
        (
            serde_json::json!({
                "title": "Newsletter!"}),
            "missing content",
        ),
    ];
    for (invalid_body, description) in test_cases {
        let response = app.post_newsletters(invalid_body).await;
        assert_eq!(
            StatusCode::UnprocessableEntity,
            response.status(),
            "The API did not return a 400 Bad Requestwhen the payload was {}.",
            description
        );
    }
}

/// Use the public API of the application under test to create
/// an unconfirmed subscriber.
async fn create_unconfirmed_subscriber(app: &TestApp) -> ConfirmationLinks {
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let _mock_guard = Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .named("Create unconfirmed subscriber")
        .expect(1)
        .mount_as_scoped(&app.email_server)
        .await;
    if !app
        .post_subscriptions(body.into())
        .await
        .status()
        .is_success()
    {
        panic!("Subscriptions request failed");
    }

    let email_request = &app
        .email_server
        .received_requests()
        .await
        .unwrap()
        .pop()
        .unwrap();
    app.get_confirmation_links(&email_request)
}

async fn create_confirmed_subscriber(app: &TestApp) {
    let confirmation_link = create_unconfirmed_subscriber(app).await;

    let client = Client::new();
    let request = surf::get(confirmation_link.html);
    if !client
        .send(request)
        .await
        .expect("Failed to execute request.")
        .status()
        .is_success()
    {
        panic!("Subscription confirmation request failed");
    }
}

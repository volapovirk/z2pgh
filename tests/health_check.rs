use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::net::TcpListener;
use uuid::Uuid;

use async_std::task;
use surf::StatusCode;
use z2pgh::configuration::{get_configuration, DatabaseSettings};
use z2pgh::startup::run;

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

async fn spawn_app() -> TestApp {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{port}");

    let mut configuration = get_configuration().expect("Failed to read configuration.");
    configuration.database.database_name = Uuid::new_v4().to_string();
    let db_pool = configure_database(&configuration.database).await;

    let _ = task::spawn(run(listener, db_pool.clone()));

    TestApp { address, db_pool }
}

pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    let mut connection = PgConnection::connect(&config.connection_string_without_db())
        .await
        .expect("Failed to connect to Postgres.");

    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database.");

    let db_pool = PgPool::connect(&config.connection_string())
        .await
        .expect("Failed to connect to Postgres.");
    sqlx::migrate!("./migrations")
        .run(&db_pool)
        .await
        .expect("Failed to migrate the database");

    db_pool
}

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
async fn subscribe_returns_a_400_when_data_is_missing() {
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

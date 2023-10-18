use chrono::Utc;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use sqlx::{PgConnection, PgPool};
use tide::convert::{Deserialize, Serialize};
use tide::Request;
use tide::Response;
use tide::StatusCode;
use uuid::Uuid;

use crate::domain::{NewSubscriber, SubscriberEmail, SubscriberName};
use crate::email_client::EmailClient;
use crate::server_state::ServerState;

#[derive(Debug, Serialize, Deserialize)]
struct FormData {
    email: String,
    name: String,
}

impl TryFrom<FormData> for NewSubscriber {
    type Error = anyhow::Error;

    fn try_from(value: FormData) -> Result<Self, Self::Error> {
        let name = SubscriberName::parse(value.name)?;
        let email = SubscriberEmail::parse(value.email)?;
        Ok(Self { email, name })
    }
}

#[tracing::instrument(
    name = "Send a confirmation email to a new subscriber",
    skip(email_client, new_subscriber, base_url, subscription_token)
)]
pub async fn send_confirmation_email(
    email_client: &EmailClient,
    new_subscriber: NewSubscriber,
    base_url: &str,
    subscription_token: &str,
) -> tide::Result<()> {
    let confirmation_link = format!(
        "{}/subscriptions/confirm?subscription_token={}",
        base_url, subscription_token
    );
    let plain_body = format!(
        "Welcome to our newsletter!\nVisit {} to confirm your subscription.",
        confirmation_link
    );
    let html_body = format!(
        "Welcome to our newsletter!< br />\
        Click <a href=\"{}\">here</a> to confirm your subscription.",
        confirmation_link
    );

    email_client
        .send_email(new_subscriber.email, "Welcome!", &html_body, &plain_body)
        .await
}

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form, db_pool, email_client, base_url),
    fields(
        request_id = %Uuid::new_v4(),
        subscriber_email = %form.email,
        subscriber_name= %form.name
    )
)]
async fn do_subscribe(
    form: FormData,
    db_pool: &PgPool,
    email_client: &EmailClient,
    base_url: &str,
) -> tide::Result {
    let new_subscriber = match form.try_into() {
        Ok(new_subscriber) => new_subscriber,
        Err(_) => return Ok(Response::new(StatusCode::BadRequest)),
    };

    let mut transaction = match db_pool.begin().await {
        Ok(transaction) => transaction,
        Err(_) => return Ok(Response::new(StatusCode::BadRequest)),
    };

    let subscriber_id = match insert_subscriber(&mut transaction, &new_subscriber).await {
        Ok(subscriber_id) => subscriber_id,
        Err(_) => return Ok(Response::new(StatusCode::InternalServerError)),
    };
    let subscription_token = generate_subscription_token();
    if store_token(&mut transaction, subscriber_id, &subscription_token)
        .await
        .is_err()
    {
        return Ok(Response::new(StatusCode::BadRequest));
    }

    if send_confirmation_email(email_client, new_subscriber, base_url, &subscription_token)
        .await
        .is_err()
    {
        return Ok(Response::new(StatusCode::InternalServerError));
    }

    if transaction.commit().await.is_err() {
        return Ok(Response::new(StatusCode::InternalServerError));
    }

    Ok(Response::new(StatusCode::Ok))
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(new_subscriber, db_connection)
)]
async fn insert_subscriber(
    db_connection: &mut PgConnection,
    new_subscriber: &NewSubscriber,
) -> Result<Uuid, sqlx::Error> {
    let subscriber_id = Uuid::new_v4();
    sqlx::query!(
        r#"
    INSERT INTO subscriptions (id, email, name, subscribed_at, status)
    VALUES ($1, $2, $3, $4, 'pending_confirmation')
    "#,
        subscriber_id,
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
        Utc::now()
    )
    .execute(db_connection)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
        // Using the `?` operator to return early
        // if the function failed, returning a sqlx::Error
        // We will talk about error handling in depth later!
    })?;
    Ok(subscriber_id)
}

#[tracing::instrument(
    name = "Store subscription token in the database",
    skip(subscription_token, db_connection)
)]
async fn store_token(
    db_connection: &mut PgConnection,
    subscriber_id: Uuid,
    subscription_token: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
    INSERT INTO subscription_tokens (subscription_token, subscriber_id)
    VALUES ($1, $2)
    "#,
        subscription_token,
        subscriber_id,
    )
    .execute(db_connection)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
        // Using the `?` operator to return early
        // if the function failed, returning a sqlx::Error
        // We will talk about error handling in depth later!
    })?;
    Ok(())
}

pub async fn subscribe(mut req: Request<ServerState>) -> tide::Result {
    let form: FormData = req.body_form().await?;
    let state = req.state();
    do_subscribe(form, &state.db_pool, &state.email_client, &state.base_url).await
}

fn generate_subscription_token() -> String {
    let mut rng = thread_rng();
    std::iter::repeat_with(|| rng.sample(Alphanumeric))
        .map(char::from)
        .take(25)
        .collect()
}

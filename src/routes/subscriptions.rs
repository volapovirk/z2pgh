use anyhow::Context;
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
    type Error = String;

    fn try_from(value: FormData) -> Result<Self, Self::Error> {
        let name = SubscriberName::parse(value.name)?;
        let email = SubscriberEmail::parse(value.email)?;
        Ok(Self { email, name })
    }
}

#[derive(thiserror::Error)]
pub enum SubscribeError {
    #[error("{0}")]
    ValidationError(String),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for SubscribeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
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
) -> anyhow::Result<()> {
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
        .map_err(surf::Error::into_inner)
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
) -> Result<(), SubscribeError> {
    let new_subscriber = form.try_into().map_err(SubscribeError::ValidationError)?;

    let mut transaction = db_pool
        .begin()
        .await
        .context("Failed to acquire a Postgress connection from the pool")?;

    let subscriber_id = insert_subscriber(&mut transaction, &new_subscriber)
        .await
        .context("Failed to insert new subsriber in the database")?;
    let subscription_token = generate_subscription_token();
    store_token(&mut transaction, subscriber_id, &subscription_token)
        .await
        .context("Failed to store the confirmation token for a new subscriber.")?;
    transaction
        .commit()
        .await
        .context("Failed to commit SQL transaction to store a new subscriber.")?;

    send_confirmation_email(email_client, new_subscriber, base_url, &subscription_token)
        .await
        .context("Failed to send a confirmation email.")?;

    Ok(())
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
    .await?;
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
    .await?;
    Ok(())
}

pub async fn subscribe(mut req: Request<ServerState>) -> tide::Result {
    let form: FormData = req.body_form().await?;
    let state = req.state();
    match do_subscribe(form, &state.db_pool, &state.email_client, &state.base_url).await {
        Ok(_) => Ok(Response::new(StatusCode::Ok)),
        Err(e) => match e {
            SubscribeError::ValidationError(s) => {
                Err(tide::Error::new(StatusCode::BadRequest, anyhow::anyhow!(s)))
            }
            SubscribeError::UnexpectedError(e) => {
                Err(tide::Error::new(StatusCode::InternalServerError, e))
            }
        },
    }
}

fn generate_subscription_token() -> String {
    let mut rng = thread_rng();
    std::iter::repeat_with(|| rng.sample(Alphanumeric))
        .map(char::from)
        .take(25)
        .collect()
}

fn error_chain_fmt(
    e: &impl std::error::Error,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    writeln!(f, "{}\n", e)?;
    let mut current = e.source();
    while let Some(cause) = current {
        writeln!(f, "Caused by:\n\t{}", cause)?;
        current = cause.source();
    }
    Ok(())
}

use crate::domain::SubscriberEmail;
use crate::email_client::EmailClient;
use crate::routes::error_chain_fmt;
use anyhow::Context;
use sqlx::PgPool;
use tide::Request;
use tide::Response;
use tide::StatusCode;

use crate::server_state::ServerState;

#[derive(serde::Deserialize)]
pub struct BodyData {
    title: String,
    content: Content,
}

#[derive(serde::Deserialize)]
pub struct Content {
    html: String,
    text: String,
}

struct ConfirmedSubscriber {
    email: SubscriberEmail,
}

#[derive(thiserror::Error)]
pub enum PublishError {
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}
// Same logic to get the full error chain on `Debug`
impl std::fmt::Debug for PublishError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

#[tracing::instrument(name = "Get confirmed subscribers", skip(pool))]
async fn get_confirmed_subscribers(
    pool: &PgPool,
) -> Result<Vec<Result<ConfirmedSubscriber, anyhow::Error>>, anyhow::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT email
        FROM subscriptions
        WHERE status = 'confirmed'
        "#,
    )
    .fetch_all(pool)
    .await?;
    let confirmed_subscribers = rows
        .into_iter()
        .map(|r| match SubscriberEmail::parse(r.email) {
            Ok(email) => Ok(ConfirmedSubscriber { email }),
            Err(error) => anyhow::bail!(error),
        })
        .collect();
    Ok(confirmed_subscribers)
}

async fn do_publish_newsletter(
    body: BodyData,
    db_pool: &PgPool,
    email_client: &EmailClient,
) -> Result<(), PublishError> {
    let subscribers = get_confirmed_subscribers(db_pool).await?;
    for subscriber in subscribers {
        match subscriber {
            Ok(subscriber) => {
                email_client
                    .send_email(
                        &subscriber.email,
                        &body.title,
                        &body.content.html,
                        &body.content.text,
                    )
                    .await
                    .map_err(|e| e.into_inner())
                    .with_context(|| {
                        format!("Failed to send newsletter issue to {}", subscriber.email)
                    })?;
            }
            Err(error) => {
                tracing::warn!(
                    error.cause_chain = ?error,
                    "Skipping a confirmed subscriber. \
                    Their stored contact details are invalid",
                );
            }
        }
    }
    Ok(())
}

pub async fn publish_newsletter(mut req: Request<ServerState>) -> tide::Result {
    let body: BodyData = req.body_json().await?;
    let state = req.state();
    match do_publish_newsletter(body, &state.db_pool, &state.email_client).await {
        Ok(_) => Ok(Response::new(StatusCode::Ok)),
        Err(e) => Err(tide::Error::new(StatusCode::InternalServerError, e)),
    }
}

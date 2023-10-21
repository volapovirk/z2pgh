use sqlx::PgPool;
use tide::Error;
use tide::Request;
use tide::Response;
use tide::StatusCode;
use uuid::Uuid;

use crate::server_state::ServerState;

#[derive(serde::Deserialize)]
pub struct Parameters {
    subscription_token: String,
}

#[tracing::instrument(name = "Confirm a pending subscriber", skip(req))]
pub async fn confirm(req: Request<ServerState>) -> tide::Result {
    let param: Parameters = req.query()?;
    let pool = &req.state().db_pool;
    let id = match get_subscriber_id_from_token(pool, &param.subscription_token).await {
        Ok(id) => id,
        Err(e) => return Err(Error::new(StatusCode::InternalServerError, e)),
    };
    match id {
        None => Ok(Response::new(StatusCode::Unauthorized)),
        Some(subscriber_id) => {
            if let Err(e) = confirm_subscriber(pool, subscriber_id).await {
                return Err(Error::new(StatusCode::InternalServerError, e));
            }
            Ok(Response::new(StatusCode::Ok))
        }
    }
}

#[tracing::instrument(name = "Mark subscriber as confirmed", skip(subscriber_id, pool))]
pub async fn confirm_subscriber(pool: &PgPool, subscriber_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"UPDATE subscriptions SET status = 'confirmed' WHERE id = $1"#,
        subscriber_id,
    )
    .execute(pool)
    .await?;
    Ok(())
}
#[tracing::instrument(name = "Get subscriber_id from token", skip(subscription_token, pool))]
pub async fn get_subscriber_id_from_token(
    pool: &PgPool,
    subscription_token: &str,
) -> Result<Option<Uuid>, sqlx::Error> {
    let result = sqlx::query!(
        r#"SELECT subscriber_id FROM subscription_tokens WHERE subscription_token = $1"#,
        subscription_token,
    )
    .fetch_optional(pool)
    .await?;
    Ok(result.map(|r| r.subscriber_id))
}

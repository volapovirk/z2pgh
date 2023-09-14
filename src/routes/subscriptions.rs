use chrono::Utc;
use sqlx::PgPool;
use tide::convert::{Deserialize, Serialize};
use tide::Request;
use tide::Response;
use tide::StatusCode;
use uuid::Uuid;

use crate::state::State;

#[derive(Debug, Serialize, Deserialize)]
struct FormData {
    email: String,
    name: String,
}

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form, db_pool),
    fields(
        request_id = %Uuid::new_v4(),
        subscriber_email = %form.email,
        subscriber_name= %form.name
    )
)]
async fn do_subscribe(form: FormData, db_pool: &PgPool) -> tide::Result {
    match insert_subscriber(db_pool, &form).await {
        Ok(_) => Ok(Response::new(StatusCode::Ok)),
        Err(e) => {
            tracing::error!("Failed to execute query: {}", e);
            Ok(Response::new(StatusCode::InternalServerError))
        }
    }
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(form, pool)
)]
async fn insert_subscriber(pool: &PgPool, form: &FormData) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
    INSERT INTO subscriptions (id, email, name, subscribed_at)
    VALUES ($1, $2, $3, $4)
    "#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    )
    .execute(pool)
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

pub async fn subscribe(mut req: Request<State>) -> tide::Result {
    let form: FormData = req.body_form().await?;
    do_subscribe(form, req.state().db_pool.as_ref()).await
}

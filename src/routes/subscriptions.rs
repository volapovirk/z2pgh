use chrono::Utc;
use tide::convert::{Deserialize, Serialize};
use tide::Request;
use tide::Response;
use tide::StatusCode;
use tracing::Instrument;
use uuid::Uuid;

use crate::state::State;

#[derive(Debug, Serialize, Deserialize)]
struct FormData {
    email: String,
    name: String,
}

pub async fn subscribe(mut req: Request<State>) -> tide::Result {
    let request_id = Uuid::new_v4();
    let data: FormData = req.body_form().await?;
    let request_span = tracing::info_span!(
        "Adding a new subscriber.",
        %request_id,
        subscriber_email = %data.email,
        subscriber_name= %data.name
    );
    let _request_span_guard = request_span.enter();

    let query_span = tracing::info_span!("Saving new subscriber details in the database");

    match sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        data.email,
        data.name,
        Utc::now()
    )
    .execute(req.state().db_pool.as_ref())
    .instrument(query_span)
    .await
    {
        Ok(_) => Ok(Response::new(StatusCode::Ok)),
        Err(e) => {
            tracing::error!("Failed to execute query: {}", e);
            Ok(Response::new(StatusCode::InternalServerError))
        }
    }
}

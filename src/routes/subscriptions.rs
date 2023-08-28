use chrono::Utc;
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

pub async fn subscribe(mut req: Request<State>) -> tide::Result {
    let data: FormData = req.body_form().await?;
    sqlx::query!(
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
    .await?;
    Ok(Response::new(StatusCode::Ok))
}

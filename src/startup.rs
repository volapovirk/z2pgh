use std::net::TcpListener;
use tide_tracing::TraceMiddleware;

use sqlx::PgPool;

use crate::{
    email_client::EmailClient,
    routes::{health_check, subscribe},
    state::State,
};

pub async fn run(
    listener: TcpListener,
    db_pool: PgPool,
    email_client: EmailClient,
) -> std::io::Result<()> {
    let connection = State::new(db_pool, email_client);
    let mut app = tide::with_state(connection);
    app.with(TraceMiddleware::new());
    app.at("/health_check").get(health_check);
    app.at("/subscriptions").post(subscribe);
    app.listen(listener).await
}

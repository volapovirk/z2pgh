use std::net::TcpListener;
use tide_tracing::TraceMiddleware;

use sqlx::PgPool;

use crate::{
    routes::{health_check, subscribe},
    state::State,
};

pub async fn run(listener: TcpListener, db_pool: PgPool) -> std::io::Result<()> {
    let connection = State::new(db_pool);
    let mut app = tide::with_state(connection);
    app.with(TraceMiddleware::new());
    app.at("/health_check").get(health_check);
    app.at("/subscriptions").post(subscribe);
    app.listen(listener).await
}

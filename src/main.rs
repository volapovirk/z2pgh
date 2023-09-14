use sqlx::PgPool;
use std::net::TcpListener;
use z2pgh::configuration::get_configuration;
use z2pgh::startup::run;
use z2pgh::telemetry::{get_subscriber, init_subscriber};

#[async_std::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("z2pgh".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration.");
    let db_pool = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to Postgres.");
    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(address)?;
    run(listener, db_pool).await
}

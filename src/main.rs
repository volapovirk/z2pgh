use sqlx::PgPool;
use std::net::TcpListener;
use z2pgh::configuration::get_configuration;
use z2pgh::startup::run;

#[async_std::main]
async fn main() -> std::io::Result<()> {
    let configuration = get_configuration().expect("Failed to read configuration.");
    let db_pool = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to Postgres.");
    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(address)?;
    run(listener, db_pool).await
}

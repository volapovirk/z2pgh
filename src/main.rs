use sqlx::postgres::PgPoolOptions;
use std::net::TcpListener;
use z2pgh::configuration::get_configuration;
use z2pgh::startup::run;
use z2pgh::telemetry::{get_subscriber, init_subscriber};

#[async_std::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("z2pgh".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration.");
    let db_pool = PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(configuration.database.with_db());
    let address = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    );
    let listener = TcpListener::bind(address)?;
    run(listener, db_pool).await
}

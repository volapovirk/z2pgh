use std::net::TcpListener;
use tide::listener::ToListener;
use tide::prelude::Listener;
use tide_tracing::TraceMiddleware;

use crate::configuration::{DatabaseSettings, Settings};
use crate::server_state::ServerState;
use crate::{
    email_client::EmailClient,
    routes::{health_check, subscribe},
};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

type ServerListener = <TcpListener as ToListener<ServerState>>::Listener;

pub struct Application {
    port: u16,
    listener: ServerListener,
}

impl Application {
    pub async fn build(configuration: Settings) -> std::io::Result<Self> {
        let connection_pool = get_connection_pool(&configuration.database);

        let sender_email = configuration
            .email_client
            .sender()
            .expect("Invalid sender email address.");
        let timeout = configuration.email_client.timeout();
        let email_client = EmailClient::new(
            configuration.email_client.base_url,
            sender_email,
            configuration.email_client.authorization_token,
            timeout,
        );

        let address = format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        );
        let listener = TcpListener::bind(address)?;
        let port = listener.local_addr().unwrap().port();
        let listener = run(listener, connection_pool, email_client).await?;
        Ok(Self { port, listener })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(mut self) -> std::io::Result<()> {
        self.listener.accept().await
    }
}

pub fn get_connection_pool(configuration: &DatabaseSettings) -> PgPool {
    PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(configuration.with_db())
}

pub async fn run(
    listener: TcpListener,
    db_pool: PgPool,
    email_client: EmailClient,
) -> std::io::Result<ServerListener> {
    let state = ServerState::new(db_pool, email_client);
    let mut app = tide::with_state(state);
    app.with(TraceMiddleware::new());
    app.at("/health_check").get(health_check);
    app.at("/subscriptions").post(subscribe);
    app.bind(listener).await
}

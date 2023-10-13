use z2pgh::configuration::get_configuration;
use z2pgh::startup::Application;
use z2pgh::telemetry::{get_subscriber, init_subscriber};

#[async_std::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("z2pgh".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration.");
    let application = Application::build(configuration).await?;
    application.run_until_stopped().await
}

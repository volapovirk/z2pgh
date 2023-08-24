use std::net::TcpListener;

use crate::routes::*;

pub async fn run(listener: TcpListener) -> std::io::Result<()> {
    let mut app = tide::new();
    app.at("/health_check").get(health_check);
    app.at("/subscriptions").post(subscribe);
    app.listen(listener).await
}

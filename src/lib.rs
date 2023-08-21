use std::net::TcpListener;
use tide::convert::{Deserialize, Serialize};
use tide::Request;
use tide::Response;
use tide::StatusCode;

#[derive(Debug, Serialize, Deserialize)]
struct FormData {
    email: String,
    name: String,
}

pub async fn run(listener: TcpListener) -> std::io::Result<()> {
    let mut app = tide::new();
    app.at("/health_check").get(health_check);
    app.at("/subscriptions").post(subscribe);
    app.listen(listener).await
}

async fn health_check(_req: Request<()>) -> tide::Result {
    Ok(Response::new(StatusCode::Ok))
}

async fn subscribe(mut req: Request<()>) -> tide::Result {
    let _data: FormData = req.body_form().await?;
    Ok(Response::new(StatusCode::Ok))
}

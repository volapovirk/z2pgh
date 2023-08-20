use std::net::TcpListener;
use tide::Request;
use tide::Response;
use tide::StatusCode;

pub async fn run(listener: TcpListener) -> std::io::Result<()> {
    let mut app = tide::new();
    app.at("/health_check").get(health_check);
    app.listen(listener).await
}

async fn health_check(_req: Request<()>) -> tide::Result {
    Ok(Response::new(StatusCode::Ok))
}

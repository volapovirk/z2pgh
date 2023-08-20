use tide::Request;
use tide::Response;
use tide::StatusCode;

pub async fn run(address: &str) -> std::io::Result<()> {
    let mut app = tide::new();
    app.at("/health_check").get(health_check);
    app.listen(address).await
}

async fn health_check(_req: Request<()>) -> tide::Result {
    Ok(Response::new(StatusCode::Ok))
}

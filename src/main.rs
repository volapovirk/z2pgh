use tide::Request;
use tide::Response;
use tide::StatusCode;

#[async_std::main]
async fn main() -> tide::Result<()> {
    let mut app = tide::new();
    app.at("/health_check").get(health_check);
    app.listen("127.0.0.1:8080").await?;
    Ok(())
}

async fn health_check(_req: Request<()>) -> tide::Result {
    Ok(Response::new(StatusCode::Ok))
}

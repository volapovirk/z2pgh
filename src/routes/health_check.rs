use tide::Request;
use tide::Response;
use tide::StatusCode;

pub async fn health_check(_req: Request<()>) -> tide::Result {
    Ok(Response::new(StatusCode::Ok))
}

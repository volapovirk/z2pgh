use tide::Request;
use tide::Response;
use tide::StatusCode;

use crate::state::State;

pub async fn health_check(_req: Request<State>) -> tide::Result {
    Ok(Response::new(StatusCode::Ok))
}

use tide::Request;
use tide::Response;
use tide::StatusCode;

use crate::server_state::ServerState;

pub async fn health_check(_req: Request<ServerState>) -> tide::Result {
    Ok(Response::new(StatusCode::Ok))
}

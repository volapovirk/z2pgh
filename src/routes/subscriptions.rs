use tide::convert::{Deserialize, Serialize};
use tide::Request;
use tide::Response;
use tide::StatusCode;

#[derive(Debug, Serialize, Deserialize)]
struct FormData {
    email: String,
    name: String,
}

pub async fn subscribe(mut req: Request<()>) -> tide::Result {
    let _data: FormData = req.body_form().await?;
    Ok(Response::new(StatusCode::Ok))
}

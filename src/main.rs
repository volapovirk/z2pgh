use std::net::TcpListener;
use z2pgh::run;

#[async_std::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8000")?;
    run(listener).await?;

    Ok(())
}

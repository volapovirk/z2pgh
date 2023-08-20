use z2pgh::run;

#[async_std::main]
async fn main() -> std::io::Result<()> {
    run("127.0.0.1:8000").await?;

    Ok(())
}

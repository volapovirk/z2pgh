use z2pgh::run;

#[async_std::main]
async fn main() -> std::io::Result<()> {
    run().await?;

    Ok(())
}

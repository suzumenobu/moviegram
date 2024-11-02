#[tokio::main]
async fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();
    let news = moviegram::rotten_tomatoes::fetch_news().await?;
    log::info!("News: {:#?}", news);
    Ok(())
}

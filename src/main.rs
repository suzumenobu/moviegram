use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Fetch {
        #[arg(short, long)]
        source: NewsSource,
    },
}

#[derive(ValueEnum, Clone)]
enum NewsSource {
    RottenTomatoes,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();
    let args = Args::parse();
    match args.command {
        Command::Fetch { source } => {
            let news = match source {
                NewsSource::RottenTomatoes => moviegram::rotten_tomatoes::fetch_news().await?,
            };
            println!("{}", serde_json::to_string_pretty(&news)?);
        }
    }

    Ok(())
}

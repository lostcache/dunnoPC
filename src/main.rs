use clap::Parser;

mod browser;
mod cli;
mod dispatch;
mod wm;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = cli::Args::parse();

    dispatch::action(args).await
}

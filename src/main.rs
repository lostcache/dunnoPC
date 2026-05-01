use clap::Parser;

mod browser;
mod cli;
mod wm;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = cli::Args::parse();

    match args.app {
        cli::AppCommands::Browser { cmd } => match cmd {
            cli::BrowserCommands::ListPages => {
                let pages = browser::brave::list_pages().await?;
                println!("{:?}", pages);
            }
            cli::BrowserCommands::GetPageContent => {}
            cli::BrowserCommands::FindPageElement => {}
            cli::BrowserCommands::FindPageElements => {}
            cli::BrowserCommands::PageElementInputStr => {}
            cli::BrowserCommands::OpenPage { url } => {}
            cli::BrowserCommands::ClosePage => {}
        },
    }
    Ok(())
}

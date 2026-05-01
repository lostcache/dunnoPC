use crate::browser;
use crate::cli;

pub(crate) async fn action(args: cli::Args) -> anyhow::Result<()> {
    match args.app {
        cli::AppCommands::Browser { cmd } => {
            let mut browser = browser::Chromium::connect().await?;
            match cmd {
                cli::BrowserCommands::ListPages => {
                    let pages = browser.list_pages().await?;
                    println!("{:?}", pages);
                }
                cli::BrowserCommands::GetPageContent { id, url, title } => {}
                cli::BrowserCommands::FindPageElement => {}
                cli::BrowserCommands::FindPageElements => {}
                cli::BrowserCommands::PageElementInputStr => {}
                cli::BrowserCommands::OpenPage { url } => {}
                cli::BrowserCommands::ClosePage => {}
            }
        }
    }

    Ok(())
}

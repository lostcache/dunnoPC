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
                cli::BrowserCommands::GetPageContent { id, url, title } => {
                    let content = browser
                        .get_page_content(browser::PageInfo { id, url, title })
                        .await;
                    println!("{:?}", content);
                }
                cli::BrowserCommands::FindPageElement => {}
                cli::BrowserCommands::FindPageElements => {}
                cli::BrowserCommands::PageElementInputStr {
                    id,
                    url,
                    title,
                    sel,
                    value,
                } => {
                    browser
                        .page_element_type_str(browser::PageInfo { id, url, title }, sel, value)
                        .await?;
                }
                cli::BrowserCommands::OpenPage { url } => {
                    let page = browser.open_page(&url).await?;
                    println!("{:?}", page);
                }
                cli::BrowserCommands::ClosePage { id, url, title } => {
                    browser
                        .close_page(browser::PageInfo { id, url, title })
                        .await?;
                }
                cli::BrowserCommands::NavigatePage { id, url, title, to } => {
                    browser
                        .page_navigate(browser::PageInfo { id, url, title }, &to)
                        .await?;
                }
                cli::BrowserCommands::ClickElement {
                    id,
                    url,
                    title,
                    sel,
                } => {
                    browser
                        .page_element_click(browser::PageInfo { id, url, title }, sel)
                        .await?;
                }
            }
        }
    }

    Ok(())
}

use anyhow::Context;
use chromiumoxide::Browser;
use futures::StreamExt;

/// Fetches all browser targets to filter by type page
/// Returns tuples of (target_id, title, url)
pub(crate) async fn list_pages() -> anyhow::Result<Vec<(String, String, String)>> {
    let mut browser = connect().await?;
    let browser_targets = browser
        .fetch_targets()
        .await
        .context("Couldn't fetch targets")?;
    Ok(browser_targets
        .into_iter()
        .filter(|t| t.r#type == "page")
        .map(|t| (t.target_id.as_ref().to_string(), t.url, t.title))
        .collect())
}

pub(crate) async fn connect() -> anyhow::Result<Browser> {
    let (browser, mut handler) = Browser::connect("http://localhost:9222")
        .await
        .context("Failed to connect to brave")?;
    tokio::spawn(async move { while handler.next().await.is_some() {} });
    Ok(browser)
}

pub(crate) async fn open_page(
    browser: chromiumoxide::Browser,
    page: &str,
) -> anyhow::Result<chromiumoxide::Page, anyhow::Error> {
    browser
        .new_page(page)
        .await
        .context("Coundn't open the page")
}

pub(crate) async fn get_page_content(page: chromiumoxide::Page) -> anyhow::Result<String> {
    page.content().await.context("Couldn't get page's contents")
}

pub(crate) async fn page_find_element(
    page: chromiumoxide::Page,
    sel: String,
) -> anyhow::Result<chromiumoxide::Element> {
    page.find_element(sel)
        .await
        .context("Something went wrong while finding element")
}

pub(crate) async fn page_find_elemets(
    page: chromiumoxide::Page,
    sel: String,
) -> anyhow::Result<Vec<chromiumoxide::Element>> {
    page.find_elements(sel)
        .await
        .context("Something went wrong while finding elements")
}

/// pub(crate) async fn page_element_click(
///     page: chromiumoxide::Page,
///     sel: String,
/// ) -> anyhow::Result<&chromiumoxide::Element, anyhow::Error> {
///     let ele = page_find_element(page, sel).await?;
///     ele.click()
///         .await
///         .context("Something went wrong while clicking element")
/// }

pub(crate) async fn page_element_type_str(
    page: chromiumoxide::Page,
    sel: String,
    value: String,
) -> anyhow::Result<(), anyhow::Error> {
    let ele = page_find_element(page, sel).await?;
    ele.click()
        .await?
        .type_str(value)
        .await
        .context("Something went wrong while setting element value")?;
    Ok(())
}

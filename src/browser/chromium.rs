use anyhow::Context;
use chromiumoxide::Browser;
use chromiumoxide::cdp::browser_protocol::target::TargetInfo;
use futures::StreamExt;

pub(crate) struct Chromium {
    browser: chromiumoxide::Browser,
}

#[derive(Debug)]
pub(crate) struct PageInfo {
    pub(crate) id: Option<String>,
    pub(crate) url: Option<String>,
    pub(crate) title: Option<String>,
}

impl PageInfo {
    pub(crate) fn new(
        id: Option<String>,
        url: Option<String>,
        title: Option<String>,
    ) -> anyhow::Result<Self> {
        match (&id, &url, &title) {
            (None, None, None) => anyhow::bail!("No page info provided"),
            _ => Ok(Self { id, url, title }),
        }
    }
}

impl Chromium {
    /// Connects to brave through the cli flag --remote-debugging-port=9222
    pub(crate) async fn connect() -> anyhow::Result<Self> {
        let (browser, mut handler) = Browser::connect("http://localhost:9222")
            .await
            .context("Failed to connect to brave")?;
        tokio::spawn(async move { while handler.next().await.is_some() {} });
        Ok(Self { browser })
    }

    pub(crate) async fn get_browser_targets(&mut self) -> anyhow::Result<Vec<TargetInfo>> {
        self.browser
            .fetch_targets()
            .await
            .context("Couldn't fetch targets")
    }

    /// Fetches all browser targets to filter by type page
    /// Returns tuples of (target_id, title, url)
    pub(crate) async fn list_pages(&mut self) -> anyhow::Result<Vec<PageInfo>> {
        let browser_targets = self.get_browser_targets().await?;
        browser_targets
            .into_iter()
            .filter(|t| t.r#type == "page")
            .map(|t| {
                PageInfo::new(
                    Some(t.target_id.as_ref().to_string()),
                    Some(t.url.to_string()),
                    Some(t.title.to_string()),
                )
            })
            .collect()
    }

    pub(crate) async fn get_page(
        &mut self,
        page_info: PageInfo,
    ) -> anyhow::Result<chromiumoxide::Page> {
        let browser_targets = self.get_browser_targets().await?;
        let target = browser_targets
            .into_iter()
            .filter(|t| t.r#type == "page")
            .find(
                |t| match (&page_info.id, &page_info.url, &page_info.title) {
                    (Some(id), _, _) => id.as_str() == t.target_id.as_ref(),
                    (_, Some(url), _) => url.as_str() == t.url.as_str(),
                    (_, _, Some(title)) => title.as_str() == t.title.as_str(),
                    _ => false,
                },
            )
            .context("No matching page found")?;
        self.browser
            .get_page(target.target_id)
            .await
            .context("Failed to get page from target")
    }

    pub(crate) async fn open_page(
        &mut self,
        page: &str,
    ) -> anyhow::Result<chromiumoxide::Page, anyhow::Error> {
        self.browser
            .new_page(page)
            .await
            .context("Coundn't open the page")
    }

    pub(crate) async fn get_page_content(&mut self, page_info: PageInfo) -> anyhow::Result<String> {
        let page = self.get_page(page_info).await?;
        page.content().await.context("Couldn't get page's contents")
    }

    pub(crate) async fn page_find_element(
        &mut self,
        page_info: PageInfo,
        sel: String,
    ) -> anyhow::Result<chromiumoxide::Element> {
        let page = self.get_page(page_info).await?;
        page.find_element(sel)
            .await
            .context("Something went wrong while finding element")
    }

    pub(crate) async fn page_find_elemets(
        &mut self,
        page_info: PageInfo,
        sel: String,
    ) -> anyhow::Result<Vec<chromiumoxide::Element>> {
        let page = self.get_page(page_info).await?;
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
        &mut self,
        page_info: PageInfo,
        sel: String,
        value: String,
    ) -> anyhow::Result<(), anyhow::Error> {
        let ele = self.page_find_element(page_info, sel).await?;
        ele.click()
            .await?
            .type_str(value)
            .await
            .context("Something went wrong while setting element value")?;
        Ok(())
    }
}

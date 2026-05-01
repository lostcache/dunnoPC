#[derive(clap::Parser, Debug)]
#[command(
    name = "dpc",
    author,
    version,
    about = "Make any LLM do Anything on any PC",
    long_about = "Make any LLM do Anything on any PC",
    propagate_version = true
)]
pub(crate) struct Args {
    #[command(subcommand)]
    pub(crate) app: AppCommands,
}

#[derive(clap::Subcommand, Debug)]
pub(crate) enum AppCommands {
    Browser {
        #[command(subcommand)]
        cmd: BrowserCommands,
    },
}

#[derive(clap::Subcommand, Debug)]
pub(crate) enum BrowserCommands {
    ListPages,
    GetPageContent,
    FindPageElement,
    FindPageElements,
    PageElementInputStr,
    OpenPage { url: String },
    ClosePage,
    // etc.
}

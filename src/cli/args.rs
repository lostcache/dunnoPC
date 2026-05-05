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
    GetPageContent {
        #[arg(short, long, num_args(0..=1))]
        id: Option<String>,
        #[arg(short, long, num_args(0..=1))]
        url: Option<String>,
        #[arg(short, long, num_args(0..=1))]
        title: Option<String>,
    },
    FindPageElement,
    FindPageElements,
    PageElementInputStr {
        #[arg(short, long, num_args(0..=1))]
        id: Option<String>,
        #[arg(short, long, num_args(0..=1))]
        url: Option<String>,
        #[arg(short, long, num_args(0..=1))]
        title: Option<String>,
        #[arg(short, long)]
        sel: String,
        #[arg(short, long)]
        value: String,
    },
    OpenPage {
        #[arg(short, long)]
        url: String,
    },
    ClosePage {
        #[arg(short, long, num_args(0..=1))]
        id: Option<String>,
        #[arg(short, long, num_args(0..=1))]
        url: Option<String>,
        #[arg(short, long, num_args(0..=1))]
        title: Option<String>,
    },
    NavigatePage {
        #[arg(short, long, num_args(0..=1))]
        id: Option<String>,
        #[arg(short, long, num_args(0..=1))]
        url: Option<String>,
        #[arg(short, long, num_args(0..=1))]
        title: Option<String>,
        #[arg(short = 'n', long)]
        to: String,
    },
    ClickElement {
        #[arg(short, long, num_args(0..=1))]
        id: Option<String>,
        #[arg(short, long, num_args(0..=1))]
        url: Option<String>,
        #[arg(short, long, num_args(0..=1))]
        title: Option<String>,
        #[arg(short, long)]
        sel: String,
    },
    // etc.
}

mod download;
mod error;

use download::{Downloader, StorageContext};
use structopt::StructOpt;

use crate::error::Error;

pub type Result<T, E = error::Error> = std::result::Result<T, E>;

#[derive(Clone, Debug, StructOpt)]
struct Opts {
    url: String,
    #[structopt(short, long)]
    dir: Option<String>,
    #[structopt(short, long)]
    path: Option<String>,
    #[structopt(short, long)]
    force: bool,
}

impl Opts {
    fn dir(&self) -> Option<&str> {
        self.dir.as_ref().map(AsRef::as_ref)
    }

    fn context(&self) -> StorageContext {
        self.path
            .as_ref()
            .map(|path| StorageContext::Path(path.as_ref()))
            .unwrap_or_default()
    }
}

enum Url<'a> {
    Author(&'a str),
    Story(&'a str),
    WholeStory(&'a str),
}

fn main() {
    let opts = Opts::from_args();

    if let Err(e) = run(&opts) {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}

fn run(opts: &Opts) -> Result<()> {
    let downloader = Downloader::new();
    match classify_url(&opts.url)? {
        Url::Author(url) => downloader.stories_by_author(url, opts.dir(), opts.force),
        Url::Story(url) | Url::WholeStory(url) => downloader.story(url, &opts.context(), opts.force),
    }
}

fn classify_url(url: &str) -> Result<Url> {
    if url.contains("bdsmlibrary.com/stories/author.php") {
        return Ok(Url::Author(url));
    }

    if url.contains("bdsmlibrary.com/stories/wholestory.php") {
        return Ok(Url::WholeStory(url));
    }

    if url.contains("bdsmlibrary.com/stories/story.php") {
        return Ok(Url::Story(url));
    }

    Err(Error::BadAddress(url.into()))
}

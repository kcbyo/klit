mod adapter;
mod document;
// mod download;
mod error;

use std::{
    collections::{HashMap, VecDeque},
    str::FromStr,
};

use adapter::{Adapter, BuildAdapter, BuildBdsmLibraryAdapter};
// use download::{Downloader, StorageContext};
use structopt::StructOpt;
use url::Url;

use crate::error::Error;

pub type Result<T, E = error::Error> = std::result::Result<T, E>;

#[derive(Clone, Debug, StructOpt)]
struct Opts {
    /// an item or directory to be retrieved
    url: String,
    /// a directory in which to store retrieved items
    path: Option<String>,
    /// if set, overwrite existing items
    #[structopt(short, long)]
    overwrite: bool,
}

impl Opts {
    fn domain(&self) -> Result<String> {
        Url::from_str(&self.url)?
            .domain()
            .map(Into::into)
            .ok_or_else(|| Error::MissingDomain(self.url.clone()))
    }
}

fn main() {
    let opts = Opts::from_args();

    if let Err(e) = run(&opts) {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}

fn run(opts: &Opts) -> Result<()> {
    let domain = opts.domain()?;
    let adapters = register_adapters();
    let adapter = adapters
        .get(&domain)
        .ok_or_else(|| Error::UnknownDomain(domain))?
        .build();

    let directory = dbg!(adapter.directory(&opts.url)?);

    Ok(())
}

fn register_adapters() -> HashMap<String, Box<dyn BuildAdapter + 'static>> {
    let mut map = HashMap::new();
    map.insert(
        "www.bdsmlibrary.com".into(),
        Box::new(BuildBdsmLibraryAdapter) as Box<dyn BuildAdapter + 'static>,
    );
    map
}

// fn classify_url(url: &str) -> Result<Url> {
//     if url.contains("bdsmlibrary.com/stories/author.php") {
//         return Ok(Url::Author(url));
//     }

//     if url.contains("bdsmlibrary.com/stories/wholestory.php") {
//         return Ok(Url::WholeStory(url));
//     }

//     if url.contains("bdsmlibrary.com/stories/story.php") {
//         return Ok(Url::Story(url));
//     }

//     Err(Error::BadAddress(url.into()))
// }

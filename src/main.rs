mod adapter;
mod document;
mod error;

use std::{
    borrow::Cow, collections::HashMap, fs, path::Path, str::FromStr, thread, time::Duration,
};

use adapter::BuildAdapter;
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

    /// an optional wait time (added between requests)
    #[structopt(short, long)]
    wait: Option<u64>,
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
    use owo_colors::OwoColorize;

    let domain = opts.domain()?;
    let adapters = register_adapters();
    let adapter = adapters
        .get(&*domain)
        .ok_or(Error::UnknownDomain(domain))?
        .build();

    if let Some(path) = opts.path.as_ref().map(Path::new) {
        if !path.exists() {
            fs::create_dir_all(path)?;
        }
    }

    let directory = adapter.directory(&opts.url)?;
    let mut first_iteration = true;

    for url in directory {
        if let Some(wait) = opts.wait.filter(|_| !first_iteration) {
            thread::sleep(Duration::from_secs(wait));
        }

        let url = match url {
            Ok(url) => url,
            Err(e) => {
                eprintln!("{} {}", "Warn:".yellow(), e.yellow());
                continue;
            }
        };

        let document = match adapter.download(url) {
            Ok(document) => document,
            Err(e) => {
                eprintln!("{} {}", "Warn:".yellow(), e.yellow());
                continue;
            }
        };

        let extension = document.extension();
        let filename = if !extension.is_empty() {
            document
                .title()
                .map(|title| Cow::from(sanitize_title(title)))
                .unwrap_or(Cow::Borrowed("unknown"))
                .to_string()
                + "."
                + extension
        } else {
            document
                .title()
                .map(|title| Cow::from(sanitize_title(title)))
                .unwrap_or(Cow::Borrowed("unknown"))
                .to_string()
        };

        let path = opts
            .path
            .as_ref()
            .map(|path| {
                let path = Path::new(path);
                Cow::from(path.join(&filename))
            })
            .unwrap_or_else(|| Cow::from(Path::new(&filename)));

        if !path.exists() || opts.overwrite {
            fs::write(&path, document.content())?;
            println!("{}", path.display());
        } else {
            eprintln!("warning: file exists: {}", path.display());
        }

        if first_iteration {
            first_iteration = false;
        }
    }

    Ok(())
}

fn register_adapters() -> HashMap<&'static str, Box<dyn BuildAdapter + 'static>> {
    use adapter::*;
    let mut map: HashMap<_, Box<dyn BuildAdapter + 'static>> = HashMap::new();
    map.insert("www.asstr.org", Box::new(BuildAsstrAdapter));
    map.insert("www.bdsmlibrary.com", Box::new(BuildBdsmLibraryAdapter));
    map.insert("www.sexstories.com", Box::new(BuildSexStoriesAdapter));
    map.insert("www.thefetlibrary.com", Box::new(BuildFetLibraryAdapter));
    map.insert("www.utopiastories.com", Box::new(BuildGaggedUtopiaAdapter));
    map
}

/// Remove elements of a title that cannot appear in file paths
fn sanitize_title(title: &str) -> String {
    title
        .replace(|u| u == '\\' || u == '"' || u == '?', "")
        .replace(|u| u == '/' || u == ':', "_")
}

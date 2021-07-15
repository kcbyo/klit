mod adapter;
mod document;
mod error;

use std::{borrow::Cow, collections::HashMap, fs, path::Path, str::FromStr};

use adapter::BuildAdapter;
use colored::Colorize;
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
        .get(&*domain)
        .ok_or(Error::UnknownDomain(domain))?
        .build();

    if let Some(path) = opts.path.as_ref().map(Path::new) {
        if !path.exists() {
            fs::create_dir_all(path)?;
        }
    }

    let directory = adapter.directory(&opts.url)?;
    for url in directory {
        let url = match url {
            Ok(url) => url,
            Err(e) => {
                let message = format!("Warn: {}", e);
                eprintln!("{}", message.yellow());
                continue;
            }
        };

        let document = match adapter.download(url) {
            Ok(document) => document,
            Err(e) => {
                let message = format!("Warn: {}", e);
                eprintln!("{}", message.yellow());
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

        match opts.path.as_ref() {
            Some(path) => {
                let path = Path::new(path);
                let path = path.join(&filename);
                fs::write(&path, document.content())?;
                println!("{}", path.display());
            }
            None => {
                fs::write(&filename, document.content())?;
                println!("{}", filename);
            }
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
    map
}

/// Remove elements of a title that cannot appear in file paths
fn sanitize_title(title: &str) -> String {
    title
        .replace(|u| u == '\\' || u == '"' || u == '?', "")
        .replace(|u| u == '/' || u == ':', "_")
}

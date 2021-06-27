mod adapter;
mod document;
mod error;

use std::{borrow::Cow, collections::HashMap, fs, path::Path, str::FromStr};

use adapter::{BuildAdapter, BuildBdsmLibraryAdapter};
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

        let document = match adapter.download(&url) {
            Ok(document) => document,
            Err(e) => {
                let message = format!("Warn: {}", e);
                eprintln!("{}", message.yellow());
                continue;
            }
        };

        let filename = document
            .title()
            .map(|title| Cow::from(sanitize_title(title)))
            .unwrap_or(Cow::Borrowed("unknown"))
            .to_string()
            + "."
            + document.extension();

        match opts.path.as_ref() {
            Some(path) => {
                let path = Path::new(path);
                let path = path.join(&filename);
                fs::write(path, document.content())?;
            }
            None => {
                fs::write(filename, document.content())?;
            }
        }
    }

    Ok(())
}

fn register_adapters() -> HashMap<&'static str, Box<dyn BuildAdapter + 'static>> {
    let mut map = HashMap::new();
    map.insert(
        "www.bdsmlibrary.com",
        Box::new(BuildBdsmLibraryAdapter) as Box<dyn BuildAdapter + 'static>,
    );
    map
}

/// Remove elements of a title that cannot appear in file paths
fn sanitize_title(title: &str) -> String {
    title
        .replace(|u| u == '\\' || u == '"' || u == '?', "")
        .replace(':', " -")
}

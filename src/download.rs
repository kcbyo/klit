use std::{
    fmt::Display,
    fs::{self, OpenOptions},
    io::{self, Write},
    path::PathBuf,
};

use regex::Regex;
use ureq::{Agent, AgentBuilder};

use crate::{error::Error, Result};

pub enum StorageContext<'a> {
    Dir(PathBuf),
    Path(&'a str),
    None,
}

impl StorageContext<'_> {
    pub fn write(&self, title: &str, text: &str, force: bool) -> io::Result<PathBuf> {
        match self {
            StorageContext::Dir(path) => {
                let title = sanitize_title(title);
                let mut path = path.join(title);
                path.set_extension("html");
                let mut file = OpenOptions::new().create(true).create_new(!force).write(true).open(&path)?;
                file.write_all(text.as_bytes())?;
                Ok(path)
            }

            StorageContext::Path(path) => {
                let mut file = OpenOptions::new().create(true).create_new(!force).write(true).open(&path)?;
                file.write_all(text.as_bytes())?;
                Ok(path.into())
            }

            StorageContext::None => {
                let mut path = sanitize_title(title);
                path += ".html";
                let mut file = OpenOptions::new().create(true).create_new(!force).write(true).open(&path)?;
                file.write_all(text.as_bytes())?;
                Ok(path.into())
            }
        }
    }
}

impl<'a> Default for StorageContext<'a> {
    fn default() -> Self {
        StorageContext::None
    }
}

pub struct Downloader {
    agent: Agent,
    url_factory: UrlFactory,
    title_pattern: Regex,
    story_id_pattern: Regex,
}

impl Downloader {
    pub fn new() -> Self {
        static USER_AGENT: &str =
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:88.0) Gecko/20100101 Firefox/88.0";

        Downloader {
            agent: AgentBuilder::new().user_agent(USER_AGENT).build(),
            url_factory: UrlFactory::new(),
            title_pattern: Regex::new(r"<title>([^<]+)</title>").unwrap(),
            story_id_pattern: Regex::new(r"story\.php\?storyid=(\d+)").unwrap(),
        }
    }

    pub fn stories_by_author(&self, url: &str, dir: Option<&str>, force: bool) -> Result<()> {
        let content = self.agent.get(url).call()?.into_string()?;
        let story_ids = self
            .story_id_pattern
            .captures_iter(&content)
            .flat_map(|captures| captures.get(1).map(|capture| capture.as_str()));

        let context = match dir {
            Some(dir) => {
                let dir = std::env::current_dir().and_then(|mut target_dir| {
                    target_dir.push(dir);
                    if !target_dir.exists() {
                        fs::create_dir_all(&target_dir)?;
                    }
                    Ok(target_dir)
                })?;
                StorageContext::Dir(dir)
            }
            None => StorageContext::None,
        };

        for id in story_ids {
            self.download_by_url(Url::from_id(id)?, &context, force)?;
        }

        Ok(())
    }

    pub fn story(&self, url: &str, context: &StorageContext, force: bool) -> Result<()> {
        self.download_by_url(self.url_factory.create_url(url)?, context, force)
    }

    fn download_by_url(&self, url: Url, context: &StorageContext, force: bool) -> Result<()> {
        let content = self.agent.get(&url.to_string()).call()?.into_string()?;

        // If the story doesn't end with a link to this dumbass website, it
        // probably indicates a network error of some kind. Whether or not we
        // can actually encounter such an error without ureq throwing an error,
        // I do not know.
        if content
            .rfind("MORE BDSM STORIES @ SEX STORIES POST")
            .is_none()
        {
            eprintln!("WARNING: file incomplete {}", url);
            // return Err(Error::Incomplete);
        }

        let title = self
            .title_pattern
            .captures(&content)
            .and_then(|captures| captures.get(1))
            .ok_or(Error::MissingTitle)?
            .as_str();

        let path = context.write(title, &content, force)?;
        println!("{}", path.display());
        Ok(())
    }
}

struct UrlFactory {
    pattern: Regex,
}

impl UrlFactory {
    fn new() -> Self {
        Self {
            pattern: Regex::new(
                r"www.bdsmlibrary.com/stories/.+?\?(?:authorid=(\d+)|storyid=(\d+))",
            )
            .unwrap(),
        }
    }

    fn create_url(&self, candidate: &str) -> Result<Url> {
        match self.pattern.captures(candidate) {
            Some(captures) => {
                if let Some(id) = captures.get(1) {
                    return id
                        .as_str()
                        .parse()
                        .map(Url::Author)
                        .map_err(|_| Error::BadAddress(candidate.into()));
                }

                if let Some(id) = captures.get(2) {
                    return id
                        .as_str()
                        .parse()
                        .map(Url::Story)
                        .map_err(|_| Error::BadAddress(candidate.into()));
                }

                unreachable!("matching text must include one of two capture groups");
            }
            None => Err(Error::BadAddress(candidate.into())),
        }
    }
}

enum Url {
    Author(i32),
    Story(i32),
}

impl Url {
    fn from_id(id: &str) -> Result<Self> {
        id.parse()
            .map(Url::Story)
            .map_err(|_| Error::BadId(id.into()))
    }
}

impl Display for Url {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Url::Author(id) => write!(
                f,
                "https://www.bdsmlibrary.com/stories/author.php?authorid={}",
                id
            ),

            Url::Story(id) => write!(
                f,
                "https://www.bdsmlibrary.com/stories/wholestory.php?storyid={}",
                id
            ),
        }
    }
}

/// Remove elements of a title that cannot appear in file paths
fn sanitize_title(title: &str) -> String {
    title
        .replace(|u| u == '\\' || u == '"' || u == '?', "")
        .replace(':', " -")
}

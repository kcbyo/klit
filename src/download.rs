use std::{fs, io, path::PathBuf};

use regex::Regex;
use ureq::{Agent, AgentBuilder};

use crate::{error::Error, Result};

pub enum StorageContext<'a> {
    Dir(PathBuf),
    Path(&'a str),
    None,
}

impl StorageContext<'_> {
    pub fn write(&self, title: &str, text: &str) -> io::Result<()> {
        match self {
            StorageContext::Dir(path) => {
                let title = make_safe(title);
                let mut path = path.join(title);
                path.set_extension("html");
                fs::write(path, text)
            }

            StorageContext::Path(path) => fs::write(path, text),

            StorageContext::None => {
                let mut path = make_safe(title);
                path += ".html";
                fs::write(path, text)
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
    title_pattern: Regex,
    story_id_pattern: Regex,
}

impl Downloader {
    pub fn new() -> Self {
        static USER_AGENT: &str =
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:88.0) Gecko/20100101 Firefox/88.0";

        Downloader {
            agent: AgentBuilder::new().user_agent(USER_AGENT).build(),
            title_pattern: Regex::new(r"<title>([^<]+)</title>").unwrap(),
            story_id_pattern: Regex::new(r"story\.php\?storyid=(\d+)").unwrap(),
        }
    }

    pub fn stories_by_author(&self, url: &str, dir: Option<&str>) -> Result<()> {
        let content = self.agent.get(url).call()?.into_string()?;
        let story_ids = self
            .story_id_pattern
            .captures_iter(&content)
            .flat_map(|captures| captures.get(1).map(|capture| capture.as_str()));

        let context = dir
            .and_then(|dir| {
                std::env::current_dir().ok().map(|mut current_dir| {
                    current_dir.push(dir);
                    current_dir
                })
            })
            .map(StorageContext::Dir)
            .unwrap_or_default();

        for id in story_ids {
            self.whole_story(&make_url_from_id(id), &context)?;
        }

        Ok(())
    }

    pub fn story(&self, url: &str, context: &StorageContext) -> Result<()> {
        let story_id = self
            .story_id_pattern
            .captures(url)
            .and_then(|captures| captures.get(1))
            .ok_or_else(|| Error::BadAddress(url.into()))?
            .as_str();

        self.whole_story(&make_url_from_id(story_id), &context)
    }

    pub fn whole_story(&self, url: &str, context: &StorageContext) -> Result<()> {
        let content = self.agent.get(url).call()?.into_string()?;
        let title = self
            .title_pattern
            .captures(&content)
            .and_then(|captures| captures.get(1))
            .ok_or(Error::MissingTitle)?
            .as_str();

        Ok(context.write(title, &content)?)
    }
}

fn make_url_from_id(story_id: &str) -> String {
    format!(
        "https://bdsmlibrary.com/stories/wholestory.php?storyid={}",
        story_id
    )
}

/// Remove elements of a title that cannot appear in file paths
fn make_safe(title: &str) -> String {
    title.replace("\\", "")
}

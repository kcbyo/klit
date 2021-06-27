use std::collections::HashMap;

use super::prelude::*;

pub struct BuildBdsmLibraryAdapter;

impl BuildAdapter for BuildBdsmLibraryAdapter {
    fn build(&self) -> Box<dyn Adapter + 'static> {
        Box::new(BdsmLibraryAdapter::new())
    }
}

pub struct BdsmLibraryAdapter {
    client: Client,
    author_pattern: Regex,
    story_id_pattern: Regex,
    title_pattern: Regex,
}

impl BdsmLibraryAdapter {
    fn new() -> Self {
        BdsmLibraryAdapter {
            client: Client::builder().user_agent(USER_AGENT).build().unwrap(),
            author_pattern: Regex::new(r"<title>BDSM Library - Stories by ([^<]+)</title>")
                .unwrap(),
            story_id_pattern: Regex::new(r"story\.php\?storyid=(\d+)").unwrap(),
            title_pattern: Regex::new(r"<title>([^<]+)</title>").unwrap(),
        }
    }
}

impl Default for BdsmLibraryAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl Adapter for BdsmLibraryAdapter {
    fn download(&self, url: &DocumentUrl) -> Result<Document> {
        let text = self.client.get(&url.url).send()?.text()?;
        let mut meta = url.meta.clone();
        if let Some(title) = self
            .title_pattern
            .captures(&text)
            .and_then(|x| x.get(1).map(|x| x.as_str()))
        {
            meta.insert(Meta::Title, title.into());
        }

        Ok(Document { meta, text })
    }

    fn directory(&self, url: &str) -> Result<DirectoryUrls> {
        let content = self.client.get(url).send()?.text()?;
        // let content = self.agent.get(url).call()?.into_string()?;
        let author = self
            .author_pattern
            .captures(&content)
            .and_then(|captures| captures.get(0))
            .map(|x| x.as_str().to_owned());
        let story_ids = self
            .story_id_pattern
            .captures_iter(&content)
            .flat_map(|captures| {
                captures
                    .get(1)
                    .map(|capture| StoryId(capture.as_str().into()))
            });

        let mut metadata = HashMap::new();
        if let Some(author) = author {
            metadata.insert(Meta::Author, author);
        }

        Ok(DirectoryUrls {
            urls: story_ids.map(|id| id.url()).collect(),
            page: None,
            meta: Some(Box::new(metadata)),
        })
    }
}

struct StoryId(String);

impl StoryId {
    fn url(&self) -> String {
        format!(
            "https://www.bdsmlibrary.com/stories/wholestory.php?storyid={}",
            self.0
        )
    }
}

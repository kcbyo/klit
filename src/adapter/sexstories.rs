use super::prelude::*;

pub struct BuildSexStoriesAdapter;

impl BuildAdapter for BuildSexStoriesAdapter {
    fn build(&self) -> Box<dyn Adapter + 'static> {
        Box::new(SexStoriesAdapter::new())
    }
}

pub struct SexStoriesAdapter {
    client: Client,
    title: Regex,
}

impl SexStoriesAdapter {
    fn new() -> Self {
        Self {
            client: Client::builder().user_agent(USER_AGENT).build().unwrap(),
            title: Regex::new("(.+)\\n").unwrap(),
        }
    }
}

impl Adapter for SexStoriesAdapter {
    fn directory(&self, url: &str) -> Result<DirectoryUrls> {
        let text = self.client.get(url).send()?.text()?;
        let document = nipper::Document::from(&text);

        let mut meta = HashMap::new();
        if let Some(author) = document
            .select("h3.notice > div.left")
            .iter()
            .map(|x| x.text())
            .next()
        {
            meta.insert(Meta::Author, author.to_string());
        }

        // There are, unfortunately, two nearly identical tables on this directory page.
        // The first lists the author's works, while the second lists the author's favorite
        // works by other writers. We're only interested in the first, but there's no real
        // way to tell the difference between the two--except of course that one comes first.
        // Looks like we're just going to take the "first."

        let stories = document.select("h3.notice + table").iter().next();
        let stories = stories
            .into_iter()
            .flat_map(|x| x.select("td a").iter().filter_map(|x| x.attr("href")))
            .map(RelativeUrl);

        Ok(DirectoryUrls {
            urls: stories.map(|story| story.url()).collect(),
            page: None,
            meta,
        })
    }

    fn download(&self, context: DocumentUrl) -> Result<Document> {
        let text = self.client.get(&context.url).send()?.text()?;
        let document = nipper::Document::from(&text);
        let mut meta = context.meta;
        if let Some(title) = self
            .title
            .captures(&document.select("div.story_info > h2").text())
            .and_then(|x| x.get(1))
            .map(|x| x.as_str().trim())
        {
            meta.insert(Meta::Title, title.to_string());
        }
        Ok(Document { meta, text })
    }
}

struct RelativeUrl<T>(T);

impl<T: AsRef<str>> RelativeUrl<T> {
    fn url(&self) -> String {
        format!("https://sexstories.com{}", self.0.as_ref())
    }
}

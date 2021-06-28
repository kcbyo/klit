use std::fmt::Write;

use super::prelude::*;

pub struct BuildFetLibraryAdapter;

impl BuildAdapter for BuildFetLibraryAdapter {
    fn build(&self) -> Box<dyn Adapter + 'static> {
        Box::new(FetLibraryAdapter::new())
    }
}

pub struct FetLibraryAdapter {
    client: Client,
    title: Regex,
    part: Regex,
}

impl FetLibraryAdapter {
    fn new() -> Self {
        Self {
            client: Client::builder().user_agent(USER_AGENT).build().unwrap(),
            title: Regex::new("The Fet Library :: (.+)").unwrap(),
            part: Regex::new(r"\?part=\d+").unwrap(),
        }
    }
}

impl Adapter for FetLibraryAdapter {
    fn directory(&self, url: &str) -> Result<DirectoryUrls> {
        let text = self.client.get(url).send()?.text()?;
        let document = nipper::Document::from(&text);
        let items = document
            .select("div.story-list-item > h3 > a")
            .iter()
            .filter_map(|item| item.attr("href"))
            .map(RelativeUrl);

        let mut meta = HashMap::new();
        let author = document
            .select("div.jumbotron.page-title div.container.text-center div h2")
            .text();

        if !author.is_empty() {
            meta.insert(Meta::Author, author.to_string());
        }

        Ok(DirectoryUrls {
            urls: items.map(|url| url.url()).collect(),
            page: None,
            meta,
        })
    }

    fn download(&self, context: DocumentUrl) -> Result<Document> {
        let mut meta = context.meta;
        let text = self.client.get(&context.url).send()?.text()?;

        let document = nipper::Document::from(&text);
        if let Some(title) = self
            .title
            .captures(&document.select("title").text())
            .and_then(|x| x.get(1).map(|x| x.as_str()))
        {
            meta.insert(Meta::Title, title.into());
        }

        // We use these later. Hopefully tags are per story, not per section.
        let tags = document
            .select("div.jumbotron.page-subtitle > div.container.text-center")
            .text();
        let tags = tags.trim();

        // The plan is to take ONLY the story content and generate a new document on that basis.
        let mut parts = vec![select_content(&document)];

        // When we get the initial text of the story, we also receive links to all other portions
        // of said story. Unfortunately, one of these links (the "next" link) is repeated. Having
        // said that, if we pull all but the *last* such link, we should be fine.
        let mut remaining_parts: Vec<_> = self
            .part
            .captures_iter(&text)
            .filter_map(|x| x.get(0).map(|x| x.as_str()))
            .collect();
        remaining_parts.pop();

        for part in remaining_parts {
            let url = context.url.to_string() + part;
            let text = self.client.get(&url).send()?.text()?;
            let document = nipper::Document::from(&text);
            parts.push(select_content(&document));
        }

        // Now that we have all the parts, we actually want to generate new html content. Which
        // always fucking sucks, but did you have a better idea? No? I didn't think so. Fuck off.
        let mut buf = String::new();
        writeln!(
            buf,
            "<title>{title} - {author}</title>\n<h1>{title}</h1>\n<p>By <span id=author>{author}</span></p>",
            title = meta.get(&Meta::Title).unwrap(),
            author = meta.get(&Meta::Author).unwrap()
        )
        .unwrap();
        writeln!(buf, "<p id=tags>tags: {}</p>", tags).unwrap();

        for (idx, text) in parts.into_iter().enumerate() {
            writeln!(buf, "<h2>Chapter {}</h2>\n{}", idx + 1, text).unwrap();
        }

        Ok(Document { meta, text: buf })
    }
}

fn select_content(document: &nipper::Document) -> String {
    document
        .select("div.container > div.row > div.col-12.story-content")
        .html()
        .trim()
        .to_string()
}

struct RelativeUrl<T>(T);

impl<T: AsRef<str>> RelativeUrl<T> {
    fn url(&self) -> String {
        format!("https://www.thefetlibrary.com{}", self.0.as_ref())
    }
}

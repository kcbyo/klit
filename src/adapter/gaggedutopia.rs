use super::prelude::*;

pub struct BuildGaggedUtopiaAdapter;

impl BuildAdapter for BuildGaggedUtopiaAdapter {
    fn build(&self) -> Box<dyn Adapter + 'static> {
        Box::new(GaggedUtopiaAdapter::new())
    }
}

pub struct GaggedUtopiaAdapter {
    client: Client,
    title: Regex,
}

impl GaggedUtopiaAdapter {
    fn new() -> Self {
        Self {
            client: Client::builder().user_agent(USER_AGENT).build().unwrap(),
            title: Regex::new(r#"(.+) ::"#).unwrap(),
        }
    }
}

impl Adapter for GaggedUtopiaAdapter {
    fn directory(&self, url: &str) -> Result<DirectoryUrls> {
        let text = self.client.get(url).send()?.text()?;
        let document = nipper::Document::from(&text);

        let mut meta = HashMap::new();
        if let Some(author) = try_get_author_from_url(url) {
            meta.insert(Meta::Author, author);
        }

        let link_pattern = Regex::new(r#"/code/show_story.asp/recid/\d+"#).unwrap();
        let links = document
            .select("tr > td > b > a")
            .iter()
            .filter_map(|cx| cx.attr("href"))
            .filter(|x| link_pattern.is_match(x))
            .map(RelativeUrl);

        Ok(DirectoryUrls {
            urls: links.map(|link| link.url()).collect(),
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
            .captures(try_get_title(&document).as_ref())
            .and_then(|cx| cx.get(1))
            .map(|cx| cx.as_str().to_owned())
        {
            meta.insert(Meta::Title, title);
        }

        Ok(Document { meta, text })
    }
}

fn try_get_author_from_url(url: &str) -> Option<String> {
    // https://www.utopiastories.com/code/show_result.asp?search=basic&author=Roger
    let pattern = Regex::new(r#"author=([^&]+)"#).unwrap();
    pattern
        .captures(url)
        .and_then(|cx| cx.get(1))
        .map(|cx| cx.as_str().to_owned())
}

fn try_get_title(document: &nipper::Document) -> impl AsRef<str> {
    document.select("head > title").text()
}

struct RelativeUrl<T>(T);

impl<T: AsRef<str>> RelativeUrl<T> {
    fn url(&self) -> String {
        format!("https://www.utopiastories.com{}", self.0.as_ref())
    }
}

use super::prelude::*;

pub struct BuildAsstrAdapter;

impl BuildAdapter for BuildAsstrAdapter {
    fn build(&self) -> Box<dyn Adapter + 'static> {
        Box::new(AsstrAdapter::new())
    }
}

pub struct AsstrAdapter {
    client: Client,
}

impl AsstrAdapter {
    fn new() -> Self {
        Self {
            client: Client::builder().user_agent(USER_AGENT).build().unwrap(),
        }
    }
}

impl Adapter for AsstrAdapter {
    fn directory(&self, url: &str) -> Result<DirectoryUrls> {
        let text = self.client.get(url).send()?.text()?;
        let document = nipper::Document::from(&text);

        // The first item is "../" and actually just goes up one directory. I could just
        // call .skip, but what if I run into a listing where that's not present?!
        let items = document
            .select("td.link > a")
            .iter()
            .filter_map(|item| item.attr("href"))
            .filter(|link| "../" != link.as_ref());

        Ok(DirectoryUrls {
            urls: items
                .map(|route| url.to_string() + route.as_ref())
                .collect(),
            page: None,
            meta: HashMap::new(),
        })
    }

    fn download(&self, context: DocumentUrl) -> Result<Document> {
        let text = self.client.get(&context.url).send()?.text()?;
        let mut meta = context.meta;
        let name = name_from_url(&context.url);
        meta.insert(Meta::Title, name.into());
        Ok(Document { meta, text })
    }
}

fn name_from_url(url: &str) -> &str {
    let left = url.rfind('/').map(|idx| idx + 1).unwrap_or_default();
    let right = url.rfind('?').unwrap_or(url.len());
    &url[left..right]
}

#[cfg(test)]
mod tests {
    #[test]
    fn name_from_url() {
        let actual = super::name_from_url("https://www.asstr.org/files/Collections/Old_Joe's_Collection/Rape/Dark_Dreamer/SORO-SLV.002");
        assert_eq!("SORO-SLV.002", actual);
    }
}

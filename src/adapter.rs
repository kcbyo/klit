mod bdsmlibrary;

use std::collections::{HashMap, VecDeque};

use crate::{
    document::{Document, Meta},
    Result,
};

pub use bdsmlibrary::{BdsmLibraryAdapter, BuildBdsmLibraryAdapter};

pub struct DocumentUrl {
    meta: HashMap<Meta, String>,
    url: String,
}

pub struct DirectoryUrls {
    urls: VecDeque<String>,
    page: Option<Box<dyn Paging + 'static>>,
    meta: Option<Box<dyn MetaSource + 'static>>,
}

impl std::fmt::Debug for DirectoryUrls {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DirectoryUrls")
            .field("urls", &self.urls)
            .field(
                "page",
                &if self.page.is_some() {
                    "Some(Page)"
                } else {
                    "None"
                },
            )
            .field(
                "meta",
                &if self.meta.is_some() {
                    "Some(Meta)"
                } else {
                    "None"
                },
            )
            .finish()
    }
}

pub trait Paging {
    fn next_page(&mut self) -> Option<Result<VecDeque<String>>>;
}

pub trait MetaSource {
    fn apply_metadata(&self, url: String) -> DocumentUrl;
}

impl MetaSource for HashMap<Meta, String> {
    fn apply_metadata(&self, url: String) -> DocumentUrl {
        DocumentUrl {
            meta: self.clone(),
            url,
        }
    }
}

impl Iterator for DirectoryUrls {
    type Item = Result<DocumentUrl>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.urls.is_empty() {
            match self.page.as_mut()?.next_page()? {
                Ok(urls) => self.urls = urls,
                Err(e) => return Some(Err(e)),
            }
            return self.next();
        }

        self.urls
            .pop_front()
            .and_then(|url| self.meta.as_ref().map(|meta| meta.apply_metadata(url)))
            .map(Ok)
    }
}

pub trait BuildAdapter {
    fn build(&self) -> Box<dyn Adapter + 'static>;
}

pub trait Adapter {
    fn document(&self, url: &str) -> Result<Document>;
    fn directory(&self, url: &str) -> Result<DirectoryUrls>;
}

mod prelude {
    pub static USER_AGENT: &str = "";
    pub use super::{Adapter, BuildAdapter, DirectoryUrls, MetaSource};
    pub use crate::{
        document::{Document, Meta},
        Result,
    };
    pub use regex::{Regex, RegexBuilder};
    pub use std::collections::HashMap;
    pub use ureq::{Agent, AgentBuilder};
}

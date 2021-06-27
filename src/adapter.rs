mod bdsmlibrary;
mod sexstories;

use std::collections::{HashMap, VecDeque};

use crate::{
    document::{Document, Meta},
    Result,
};

pub use bdsmlibrary::{BdsmLibraryAdapter, BuildBdsmLibraryAdapter};
pub use sexstories::{BuildSexStoriesAdapter, SexStoriesAdapter};

pub struct DocumentUrl {
    meta: HashMap<Meta, String>,
    url: String,
}

pub struct DirectoryUrls {
    urls: VecDeque<String>,
    page: Option<Box<dyn Paging + 'static>>,
    meta: HashMap<Meta, String>,
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
            .field("meta", &self.meta)
            .finish()
    }
}

pub trait Paging {
    fn next_page(&mut self) -> Option<Result<VecDeque<String>>>;
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
            .map(|url| DocumentUrl {
                meta: self.meta.clone(),
                url,
            })
            .map(Ok)
    }
}

pub trait BuildAdapter {
    fn build(&self) -> Box<dyn Adapter + 'static>;
}

pub trait Adapter {
    fn directory(&self, url: &str) -> Result<DirectoryUrls>;
    fn download(&self, url: &DocumentUrl) -> Result<Document>;
}

mod prelude {
    pub static USER_AGENT: &str = "";
    pub use super::{Adapter, BuildAdapter, DirectoryUrls, DocumentUrl};
    pub use crate::{
        document::{Document, Meta},
        Result,
    };
    pub use regex::{Regex, RegexBuilder};
    pub use reqwest::blocking::Client;
    pub use std::collections::HashMap;
}

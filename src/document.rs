use std::collections::HashMap;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Meta {
    Author,
    Extension,
    Other(String),
    PublicationDate,
    Title,
}

pub struct Document {
    pub meta: HashMap<Meta, String>,
    pub text: String,
}

impl Document {
    pub fn author(&self) -> Option<&str> {
        self.meta.get(&Meta::Author).map(AsRef::as_ref)
    }

    pub fn content(&self) -> &str {
        &self.text
    }

    pub fn extension(&self) -> &str {
        self.meta
            .get(&Meta::Extension)
            .map(AsRef::as_ref)
            .unwrap_or("html")
    }

    pub fn publication_date(&self) -> Option<&str> {
        self.meta.get(&Meta::PublicationDate).map(AsRef::as_ref)
    }

    pub fn title(&self) -> Option<&str> {
        self.meta.get(&Meta::Title).map(AsRef::as_ref)
    }
}

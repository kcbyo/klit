use std::collections::HashMap;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Meta {
    Title,
    Author,
    PublicationDate,
    Other(String),
}

pub struct Document {
    meta: HashMap<Meta, String>,
    text: String,
}

impl Document {
    pub fn title(&self) -> Option<&str> {
        self.meta.get(&Meta::Title).map(AsRef::as_ref)
    }

    pub fn author(&self) -> Option<&str> {
        self.meta.get(&Meta::Author).map(AsRef::as_ref)
    }

    pub fn publication_date(&self) -> Option<&str> {
        self.meta.get(&Meta::PublicationDate).map(AsRef::as_ref)
    }

    pub fn content(&self) -> &str {
        &self.text
    }
}

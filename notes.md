# Design notes

Looking for a way to define a story downloader as a set of traits. That's always tough to do, of course, but we'll get something done. Right? ...Right? I think the easiest thing would be to accept a URL and return a string, or possibly a set of strings.

```rust
struct Document {
    title: Option<String>,
    author: Option<String>,
    text: String,
}

struct DocumentUrls;

impl Iterator for DocumentUrls {
    Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

trait ArchiveService {
    fn document(&self, url: &str) -> Document;
    fn directory(&self, url: &str) -> DocumentUrls;
}
```

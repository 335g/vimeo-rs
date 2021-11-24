use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Segment {
    end: f64,
    start: f64,
    url: String,
}

impl Segment {
    pub fn url(&self) -> &str {
        self.url.as_str()
    }
}
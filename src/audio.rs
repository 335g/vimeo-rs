use reqwest::Url;
use serde::Deserialize;
use async_trait::async_trait;
use crate::segment::Segment;
use crate::get::Get;
use crate::error::VimeoError;

#[derive(Debug, Deserialize)]
pub struct Audio {
    base_url: String,
    init_segment: String,
    segments: Vec<Segment>
}

#[async_trait]
impl Get for Audio {
    fn init_segment(&self) -> &str {
        self.init_segment.as_str()
    }

    fn segments(&self) -> &[Segment] {
        &self.segments
    }

    fn url(&self, base_url: &Url) -> Result<Url, VimeoError> {
        let (_, url) = self.base_url.split_at(3);
        let url = base_url.join(url)?;

        Ok(url)
    }
}
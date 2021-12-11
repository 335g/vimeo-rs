use reqwest::Url;
use serde::Deserialize;
use async_trait::async_trait;
use crate::error::VimeoError;
use crate::segment::Segment;
use crate::get::Get;

#[derive(Debug, Deserialize)]
pub struct Video {
    height: f64,
    base_url: String,
    init_segment: String,
    segments: Vec<Segment>,
}

impl Video {
    pub fn height(&self) -> f64 {
        self.height
    }
}

#[async_trait]
impl Get for Video {
    fn init_segment(&self) -> &str {
        self.init_segment.as_str()
    }

    fn segments(&self) -> &[Segment] {
        &self.segments
    }

    fn url(&self, base_url: &Url) -> Result<Url, VimeoError> {
        let url = base_url.join(&format!("video/{}", &self.base_url))?;

        Ok(url)
    }

    
}
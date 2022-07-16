use reqwest::Url;
use serde::Deserialize;
use async_trait::async_trait;
use crate::segment::Segment;
use crate::get::Get;
use crate::error::VimeoError;

#[readonly::make]
#[derive(Debug, Deserialize)]
pub struct Audio {
    pub id: String,
    pub base_url: String,
    pub format: String,
    pub mime_type: String,
    pub codecs: String,
    pub bitrate: usize,
    pub avg_bitrate: usize,
    pub duration: f32,
    pub channels: usize,
    pub sample_rate: usize,
    pub max_segment_duration: usize,
    pub init_segment: String,
    pub index_segment: String,
    pub segments: Vec<Segment>,
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
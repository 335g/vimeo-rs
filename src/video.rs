use reqwest::Url;
use serde::Deserialize;
use async_trait::async_trait;
use crate::error::VimeoError;
use crate::segment::Segment;
use crate::get::Get;

#[readonly::make]
#[derive(Debug, Deserialize)]
pub struct Video {
    pub id: String,
    pub base_url: String,
    pub format: String,
    pub mime_type: String,
    pub codecs: String,
    pub bitrate: usize,
    pub avg_bitrate: usize,
    pub duration: f32,
    pub framerate: f32,
    pub width: usize,
    pub height: usize,
    pub max_segment_duration: usize,
    pub init_segment: String,
    pub index_segment: Option<String>,
    pub segments: Vec<Segment>,
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
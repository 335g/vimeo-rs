use serde::Deserialize;
use async_trait::async_trait;
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

    #[allow(dead_code)]
    pub fn base_url(&self) -> &str {
        &self.base_url
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
}
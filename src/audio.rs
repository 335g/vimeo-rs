use serde::Deserialize;
use async_trait::async_trait;
use crate::segment::Segment;
use crate::get::Get;

#[derive(Debug, Deserialize)]
pub struct Audio {
    base_url: String,
    init_segment: String,
    segments: Vec<Segment>
}

impl Audio {
    pub fn base_url(&self) -> &str {
        &self.base_url
    }
}

#[async_trait]
impl Get for Audio {
    fn init_segment(&self) -> &str {
        self.init_segment.as_str()
    }

    fn segments(&self) -> &[Segment] {
        &self.segments
    }
}
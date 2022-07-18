use serde::Deserialize;
use crate::segment::Segment;
use crate::content::Contents;

#[readonly::make]
#[derive(Debug, Clone, Deserialize)]
pub struct AudioInfo {
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
    pub index_segment: Option<String>,
    pub segments: Vec<Segment>,
}

impl Contents for AudioInfo {
    fn init_segment(&self) -> &str {
        self.init_segment.as_str()
    }

    fn segments(&self) -> &[Segment] {
        &self.segments
    }
}
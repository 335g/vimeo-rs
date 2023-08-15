use serde::{Deserialize, Serialize};
use crate::content::{Segment, Assemblable};

#[readonly::make]
#[derive(Debug, Deserialize, Serialize)]
pub struct Video {
    pub avg_bitrate: usize,
    pub base_url: String,
    pub bitrate: usize,
    pub codecs: String,
    pub duration: f32,
    pub format: String,
    pub framerate: f32,
    pub height: usize,
    pub id: String,
    pub index_segment: String,
    pub init_segment: String,
    pub max_segment_duration: usize,
    pub mime_type: String,
    pub segments: Vec<Segment>,
    pub width: usize,
}

impl Video {
    pub fn expression(&self) -> VideoExp {
        VideoExp { 
            bitrate: self.bitrate, 
            codecs: self.codecs.clone(),
            framerate: self.framerate, 
            width: self.width, 
            height: self.height
        }
    }
}

impl Assemblable for Video {
    fn init_segment(&self) ->  &str {
        &self.init_segment
    }

    fn base_url(&self) ->  &str {
        &self.base_url
    }

    fn index_segment(&self) ->  &str {
        &self.index_segment
    }

    fn segments(&self) ->  &Vec<Segment>  {
        &self.segments
    }
}

#[readonly::make]
#[derive(Debug, Clone, PartialEq)]
pub struct VideoExp {
    pub bitrate: usize,
    pub codecs: String,
    pub framerate: f32,
    pub width: usize,
    pub height: usize,
}

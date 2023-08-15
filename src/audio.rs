use serde::{Deserialize, Serialize};
use crate::content::{Segment, Assemblable};

#[readonly::make]
#[derive(Debug, Deserialize, Serialize)]
pub struct Audio {
    pub avg_bitrate: usize,
    pub base_url: String,
    pub bitrate: usize,
    pub channels: usize,
    pub codecs: String,
    pub duration: f32,
    pub format: String,
    pub id: String,
    pub index_segment: String,
    pub init_segment: String,
    pub max_segment_duration: usize,
    pub mime_type: String,
    pub sample_rate: usize,
    pub segments: Vec<Segment>,
}

impl Audio {
    pub fn expression(&self) -> AudioExp {
        AudioExp { 
            bitrate: self.bitrate, 
            channels: self.channels, 
            codecs: self.codecs.clone(), 
            sample_rate: self.sample_rate,
        }
    }
}

impl Assemblable for Audio {
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
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct AudioExp {
    pub bitrate: usize,
    pub channels: usize,
    pub codecs: String,
    pub sample_rate: usize,
}
use serde::Deserialize;

use crate::{audio::AudioInfo, video::VideoInfo};

#[derive(Debug, Deserialize)]
pub struct ContentInfo {
    pub clip_id: String,
    pub base_url: String,

    #[serde(rename = "audio")]
    pub audio_infos: Vec<AudioInfo>,

    #[serde(rename = "video")]
    pub video_infos: Vec<VideoInfo>,
}

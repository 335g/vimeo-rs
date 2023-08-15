#![allow(dead_code)]

use base64::Engine;
use futures::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt;
use url::Url;
use async_trait::async_trait;

use crate::VimeoError;
use crate::audio::{Audio, AudioExp};
use crate::video::{Video, VideoExp};

#[derive(Debug, Deserialize)]
pub struct Content {
    base_url: String,
    audio: Vec<Audio>,
    video: Vec<Video>,
}

impl Content {
    pub fn base_url(&self) -> &String {
        &self.base_url
    }

    pub fn eq_audio(&self, exp: &AudioExp) -> Option<&Audio> {
        self.audio.iter()
            .filter(|audio| {
                exp.bitrate == audio.bitrate &&
                exp.channels == audio.channels &&
                exp.codecs == audio.codecs &&
                exp.sample_rate == audio.sample_rate
            })
            .next()
    }

    pub fn eq_video(&self, exp: &VideoExp) -> Option<&Video> {
        self.video.iter()
            .filter(|video| {
                exp.bitrate == video.bitrate &&
                exp.codecs == video.codecs &&
                exp.framerate == video.framerate &&
                exp.width == video.width &&
                exp.height == video.height
            })
            .next()
    }

    pub fn audio_exps(&self) -> Vec<AudioExp> {
        self.audio.iter()
            .map(|audio| audio.expression())
            .collect()
    }

    pub fn video_exps(&self) -> Vec<VideoExp> {
        self.video.iter()
            .map(|video| video.expression())
            .collect()
    }

    pub async fn assemble_audio(&self, audio: &Audio, base_url: Url) -> Result<Vec<u8>, VimeoError> {
        let base_url = base_url.join(&self.base_url)
            .map_err(|_| VimeoError::FailedAssembleContent { reason: "cannot join base_url - Content".to_string() })?;
        
        audio.assemble(base_url).await
    }

    pub async fn assemble_video(&self, video: &Video, base_url: Url) -> Result<Vec<u8>, VimeoError> {
        let base_url = base_url.join(&self.base_url)
            .map_err(|_| VimeoError::FailedAssembleContent { reason: "cannot join base_url - Content".to_string() })?;
        
        video.assemble(base_url).await
    }

    pub fn mp3_audios(&self) -> Vec<&Audio> {
        self.audio.iter()
            .filter(|audio| audio.codecs.starts_with("mp4a"))
            .collect()
    }
}

#[readonly::make]
#[derive(Debug, Deserialize, Serialize)]
pub struct Segment {
    start: f32,
    end: f32,
    size: usize,
    pub url: String,
}

#[async_trait]
pub trait Assemblable {
    fn init_segment(&self) -> &str;
    fn base_url(&self) -> &str;
    fn index_segment(&self) -> &str;
    fn segments(&self) -> &Vec<Segment>;

    async fn assemble(&self, base_url: Url) -> Result<Vec<u8>, VimeoError> {
        let mut buf = vec![];

        base64::engine::general_purpose::STANDARD
            .decode_vec(self.init_segment(), &mut buf)
            .map_err(|_| VimeoError::FailedAssembleContent { reason: "cannot decode init_segment - Audio".to_string() })?;

        let base_url = base_url.join(self.base_url())
            .map_err(|_| VimeoError::FailedAssembleContent { reason: "cannot join base_url - Audio".to_string() })?;
        let client = Client::builder()
            .build()
            .map_err(|_| VimeoError::FailedAssembleContent { reason: "cannot build network client - Audio".to_string() })?;
        
        // index_segment
        let url = base_url.join(self.index_segment())
            .map_err(|_| VimeoError::FailedAssembleContent { reason: "cannot join url (index_segment) - Audio".to_string() })?;
        let request = client.get(url)
            .build()
            .map_err(|_| VimeoError::FailedAssembleContent { reason: "cannot construct request (index_segment) - Audio".to_string() })?;
        let content = client.execute(request)
            .await
            .map_err(|_| VimeoError::FailedAssembleContent { reason: "cannot get response (index_segment) - Audio".to_string() })?
            .bytes()
            .await
            .map_err(|_| VimeoError::FailedAssembleContent { reason: "cannot construct byte response (index_segment) - Audio".to_string() })?;
        buf.write_all(&content)
            .await
            .map_err(|_| VimeoError::FailedAssembleContent { reason: "cannot write content to buf (index_segment) - Audio".to_string() })?;

        // segments
        let mut segments = futures::stream::iter(self.segments());
        while let Some(segment) = segments.next().await {
            let url = base_url.join(&segment.url)
                .map_err(|_| VimeoError::FailedAssembleContent { reason: "cannot join url (segments) - Audio".to_string() })?;
            let request = client.get(url)
                .build()
                .map_err(|_| VimeoError::FailedAssembleContent { reason: "cannot construct request (segments) - Audio".to_string() })?;
            let content = client.execute(request)
                .await
                .map_err(|_| VimeoError::FailedAssembleContent { reason: "cannot get response (segments) - Audio".to_string() })?
                .bytes()
                .await
                .map_err(|_| VimeoError::FailedAssembleContent { reason: "cannot construct byte response (segments) - Audio".to_string() })?;
            buf.write_all(&content)
                .await
                .map_err(|_| VimeoError::FailedAssembleContent { reason: "cannot write content to buf (segments) - Audio".to_string() })?;
        }

        Ok(buf)
    }
}
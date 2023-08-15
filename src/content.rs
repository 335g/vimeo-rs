#![allow(dead_code)]

use base64::Engine;
use futures::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use url::Url;
use tokio::io::AsyncWriteExt;

use crate::VimeoError;

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

    pub async fn assemble_audio(&self, audio: &Audio, base_url: Url, buf: &mut Vec<u8>) -> Result<(), VimeoError> {
        let base_url = base_url.join(&self.base_url)
            .map_err(|_| VimeoError::FailedAssembleContent { reason: "cannot join base_url - Content".to_string() })?;
        
        audio.assemble(base_url, buf).await?;

        Ok(())
    }

    pub async fn assemble_video(&self, video: &Video, base_url: Url, buf: &mut Vec<u8>) -> Result<(), VimeoError> {
        let base_url = base_url.join(&self.base_url)
            .map_err(|_| VimeoError::FailedAssembleContent { reason: "cannot join base_url - Content".to_string() })?;
        
        video.assemble(base_url, buf).await?;

        Ok(())
    }

    pub fn mp3_audios(&self) -> Vec<&Audio> {
        self.audio.iter()
            .filter(|audio| audio.codecs.starts_with("mp4a"))
            .collect()
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Audio {
    avg_bitrate: usize,
    base_url: String,
    bitrate: usize,
    channels: usize,
    codecs: String,
    duration: f32,
    format: String,
    id: String,
    index_segment: String,
    init_segment: String,
    max_segment_duration: usize,
    mime_type: String,
    sample_rate: usize,
    segments: Vec<Segment>,
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

    pub async fn assemble(&self, base_url: Url, buf: &mut Vec<u8>) -> Result<(), VimeoError> {
        base64::engine::general_purpose::STANDARD
            .decode_vec(&self.init_segment, buf)
            .map_err(|_| VimeoError::FailedAssembleContent { reason: "cannot decode init_segment - Audio".to_string() })?;

        let base_url = base_url.join(&self.base_url)
            .map_err(|_| VimeoError::FailedAssembleContent { reason: "cannot join base_url - Audio".to_string() })?;
        let client = Client::builder()
            .build()
            .map_err(|_| VimeoError::FailedAssembleContent { reason: "cannot build network client - Audio".to_string() })?;
        
        // index_segment
        let url = base_url.join(&self.index_segment)
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
        let mut segments = tokio_stream::iter(&self.segments);
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

        Ok(())
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Video {
    avg_bitrate: usize,
    base_url: String,
    bitrate: usize,
    codecs: String,
    duration: f32,
    format: String,
    framerate: f32,
    height: usize,
    id: String,
    index_segment: String,
    init_segment: String,
    max_segment_duration: usize,
    mime_type: String,
    segments: Vec<Segment>,
    width: usize,
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

    pub async fn assemble(&self, base_url: Url, buf: &mut Vec<u8>) -> Result<(), VimeoError> {
        base64::engine::general_purpose::STANDARD
            .decode_vec(&self.init_segment, buf)
            .map_err(|_| VimeoError::FailedAssembleContent { reason: "cannot decode init_segment - Video".to_string() })?;

        let base_url = base_url.join(&self.base_url)
            .map_err(|_| VimeoError::FailedAssembleContent { reason: "cannot join base_url - Video".to_string() })?;
        let client = Client::builder()
            .build()
            .map_err(|_| VimeoError::FailedAssembleContent { reason: "cannot build network client - Video".to_string() })?;
        
        // index_segment
        let url = base_url.join(&self.index_segment)
            .map_err(|_| VimeoError::FailedAssembleContent { reason: "cannot join url (index_segment) - Video".to_string() })?;
        let request = client.get(url)
            .build()
            .map_err(|_| VimeoError::FailedAssembleContent { reason: "cannot construct request (index_segment) - Video".to_string() })?;
        let content = client.execute(request)
            .await
            .map_err(|_| VimeoError::FailedAssembleContent { reason: "cannot get response (index_segment) - Video".to_string() })?
            .bytes()
            .await
            .map_err(|_| VimeoError::FailedAssembleContent { reason: "cannot construct byte response (index_segment) - Video".to_string() })?;
        buf.write_all(&content)
            .await
            .map_err(|_| VimeoError::FailedAssembleContent { reason: "cannot write content to buf (index_segment) - Video".to_string() })?;

        // segments
        let mut segments = tokio_stream::iter(&self.segments);
        while let Some(segment) = segments.next().await {
            let url = base_url.join(&segment.url)
                .map_err(|_| VimeoError::FailedAssembleContent { reason: "cannot join url (segments) - Video".to_string() })?;
            let request = client.get(url)
                .build()
                .map_err(|_| VimeoError::FailedAssembleContent { reason: "cannot construct request (segments) - Video".to_string() })?;
            let content = client.execute(request)
                .await
                .map_err(|_| VimeoError::FailedAssembleContent { reason: "cannot get response (segments) - Video".to_string() })?
                .bytes()
                .await
                .map_err(|_| VimeoError::FailedAssembleContent { reason: "cannot construct byte response (segments) - Video".to_string() })?;
            buf.write_all(&content)
                .await
                .map_err(|_| VimeoError::FailedAssembleContent { reason: "cannot write content to buf (segments) - Video".to_string() })?;
        }

        Ok(())
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Segment {
    start: f32,
    end: f32,
    size: usize,
    url: String,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct AudioExp {
    bitrate: usize,
    channels: usize,
    codecs: String,
    sample_rate: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VideoExp {
    bitrate: usize,
    codecs: String,
    framerate: f32,
    width: usize,
    height: usize,
}

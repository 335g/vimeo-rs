#![allow(dead_code)]

use serde::Deserialize;


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
}

#[derive(Debug, Deserialize)]
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
}

#[derive(Debug, Deserialize)]
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
}

#[derive(Debug, Deserialize)]
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

#[cfg(test)]
mod tests {
    use std::io::Read;
    use reqwest::Client;
    use crate::player_config;

    use super::*;

    #[tokio::test]
    async fn test_audio() {
        let mut f = std::fs::File::open("data/b.html").expect("is file");
        let mut html = String::new();
        f.read_to_string(&mut html).unwrap();

        let config = player_config(&html).await.expect("is player config");
        // if let Some(cdn) = config.dash_default_cdn() {
        //     let client = Client::builder().build().unwrap();
        //     let request = client.get(cdn.avc_url.clone()).build().unwrap();
        //     let content = client.execute(request)
        //         .await
        //         .expect("aaa")
        //         .json::<Content>()
        //         .await
        //         .expect("bbb");

        //     let audio1 = &content.audio[0];
        //     println!("{:?}", audio1);
        // }

        if let Some(master_url) = config.master_urls().get(0) {
            let base_url = master_url.base_url().await.expect("is master url");
            println!("{}", base_url);
        }
    }
}
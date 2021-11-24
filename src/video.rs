use http::HeaderValue;
use reqwest::Url;
use serde::Deserialize;
use async_trait::async_trait;
use tokio::io::AsyncWriteExt;
use crate::VimeoError;
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
}

#[async_trait]
impl Get for Video {
    fn init_segment(&self) -> &str {
        self.init_segment.as_str()
    }

    fn segments(&self) -> &[Segment] {
        &self.segments
    }

    async fn get<W, V>(&self, url: Url, writer: W, user_agent: V) -> Result<(), VimeoError>
    where
        W: AsyncWriteExt + Unpin + Send,
        V: TryInto<HeaderValue> + Clone + Send,
        V::Error: Into<http::Error>
    {
        let base_url = url.join(&format!("video/{}", self.base_url))?;
        self.write_segments(base_url, writer, user_agent).await?;

        Ok(())
    }
}
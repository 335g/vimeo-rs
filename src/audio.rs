use http::HeaderValue;
use reqwest::Url;
use serde::Deserialize;
use async_trait::async_trait;
use tokio::io::AsyncWriteExt;
use crate::VimeoError;
use crate::segment::Segment;
use crate::get::Get;

#[derive(Debug, Deserialize)]
pub struct Audio {
    base_url: String,
    init_segment: String,
    segments: Vec<Segment>
}

#[async_trait]
impl Get for Audio {
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
        let (_, base_url) = self.base_url.split_at(3);
        let base_url = url.join(base_url)?;
        self.write_segments(base_url, writer, user_agent).await?;

        Ok(())
    }
}
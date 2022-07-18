use std::path::Path;

use reqwest::{Client, Url};
use serde::Deserialize;
use tokio::{io::{BufWriter, AsyncWrite, AsyncWriteExt}, fs::File, sync::mpsc::Sender};

use crate::{audio::AudioInfo, video::VideoInfo, segment::Segment, VimeoError};

#[derive(Debug, Deserialize)]
pub struct ContentInfo {
    pub clip_id: String,
    pub base_url: String,

    #[serde(rename = "audio")]
    pub audio_infos: Vec<AudioInfo>,

    #[serde(rename = "video")]
    pub video_infos: Vec<VideoInfo>,
}

pub trait Contents: Sized {
    fn init_segment(&self) -> &str;
    fn segments(&self) -> &[Segment];
}

pub async fn write_segments<P, C>(contents: &C, client: &Client, base_url: Url, file_path: P, sender: Option<Sender<()>>) -> Result<(), VimeoError>
where
    P: AsRef<Path>,
    C: Contents,
{
    let f = File::create(file_path).await?;
    let mut writer = BufWriter::new(f);

    let init_segment = base64::decode(contents.init_segment())?;
    writer.write_all(&init_segment).await?;

    for seg in contents.segments() {
        let url = base_url.join(&seg.url)?;
        let resp = client.get(url)
            .send()
            .await?;

        let status = resp.status();
        if !status.is_success() {
            return Err(VimeoError::IsNotSuccess(status))
        }

        let bytes = resp.bytes().await?;
        writer.write_all(bytes.as_ref()).await?;

        #[cfg(feature = "progressbar")]
        {
            if let Some(s) = &sender {
                s.send(()).await.unwrap();
            }
        }
    }

    Ok(())
}


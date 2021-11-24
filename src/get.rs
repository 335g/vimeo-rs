
use std::path::Path;

use easy_scraper::Pattern;
use regex::Regex;
use reqwest::{IntoUrl, Url, header::HeaderValue};
use async_trait::async_trait;
use tokio::io::{AsyncWrite, AsyncWriteExt};

use crate::{audio::Audio, content::Content, error::VimeoError, segment::Segment, video::Video};

#[async_trait]
pub trait Get {
    fn init_segment(&self) -> &str;
    fn segments(&self) -> &[Segment];
    async fn get<W, V>(&self, url: Url, writer: W, user_agent: V) -> Result<(), VimeoError>
    where
        W: AsyncWriteExt + Unpin + Send,
        V: TryInto<HeaderValue> + Clone + Send,
        V::Error: Into<http::Error>;

    async fn write_segments<W, V>(&self, base_url: Url, mut writer: W, user_agent: V) -> Result<(), VimeoError>
    where
        W: AsyncWrite + Unpin + Send,
        V: TryInto<HeaderValue> + Clone + Send,
        V::Error: Into<http::Error>,
    {
        let init_segment = base64::decode(self.init_segment())?;
        writer.write_all(&init_segment).await?;
        
        for seg in self.segments() {
            let url = base_url.join(seg.url())?;
            let client = reqwest::Client::builder()
                .user_agent(user_agent.clone())
                .build()?;
            let resp = client.get(url)
                .send()
                .await?;

            let status = resp.status();
            if !status.is_success() {
                return Err(VimeoError::IsNotSuccess(status))
            }

            let bytes = resp.bytes().await?;
            writer.write_all(bytes.as_ref()).await?;
        }

        Ok(())
    }
}

async fn get_urls<U1, U2, V>(at: U1, from: U2, user_agent: V) -> Result<(Url, Url), VimeoError>
where
    U1: IntoUrl,
    HeaderValue: TryFrom<U2>,
    <HeaderValue as TryFrom<U2>>::Error: Into<http::Error>,
    V: TryInto<HeaderValue>,
    V::Error: Into<http::Error>,
{
    let client = reqwest::Client::builder()
        .user_agent(user_agent)
        .build()?;
    let req = client.get(at)
        .header("referer", from)
        .build()?;
    let resp = client.execute(req)
        .await?
        .text()
        .await?;

    let pat = Pattern::new(r#"
        <body><script>{{content}}</script></body>
    "#).unwrap();
    let master_regex = Regex::new(r#""(https://[^"]+)(video)([^"]+master.json[?][^",]+)""#).unwrap();
    let map = pat.matches(&resp)
        .into_iter()
        .filter(|m| master_regex.is_match(&m["content"]))
        .next()
        .ok_or(VimeoError::CannotDeserializeThe1stResponse)?;
    let cap = master_regex.captures(&map["content"]).unwrap();
    
    let info_url = Url::parse(&format!("{}{}{}", &cap[1], &cap[2], &cap[3]))?;
    let base_url = Url::parse(&cap[1])?;

    return Ok((info_url, base_url))
}

async fn get_audio_and_video<U, V>(url: U, user_agent: V) -> Result<(Audio, Video), VimeoError>
where
    U: IntoUrl,
    V: TryInto<HeaderValue>,
    V::Error: Into<http::Error>,
{
    let client = reqwest::Client::builder()
        .user_agent(user_agent)
        .build()?;
    let content = client.get(url)
        .send()
        .await?
        .json::<Content>()
        .await?;
    
    content.audio_and_video()
}

#[allow(dead_code)]
pub async fn get_movie<U1, U2, P, V>(at: U1, from: U2, save_file_path: P, user_agent: V) -> Result<(), VimeoError>
where
    U1: IntoUrl,
    U1: IntoUrl,
    HeaderValue: TryFrom<U2>,
    <HeaderValue as TryFrom<U2>>::Error: Into<http::Error>,
    P: AsRef<Path>,
    V: TryInto<HeaderValue> + Clone + Send + 'static,
    V::Error: Into<http::Error>,
{
    let (info_url, base_url) = get_urls(at, from, user_agent.clone()).await?;
    log::debug!("info_url: {}", &info_url);
    log::debug!("base_url: {}", &base_url);

    let (audio, video) = get_audio_and_video(info_url, user_agent.clone()).await?;
    log::debug!("audio: {:?}", &audio);
    log::debug!("video: {:?}", &video);

    let tmp_dir = tempfile::tempdir()?;
    log::debug!("tmp_dir: {:?}", &tmp_dir);

    let mp3_tmp_filepath = tmp_dir.path().join("tmp.mp3");
    let f = tokio::fs::File::create(&mp3_tmp_filepath).await?;
    let writer = tokio::io::BufWriter::new(f);
    let ua = user_agent.clone();
    let url = base_url.clone();
    let mp3_handle = tokio::spawn(async move{
        audio.get(url, writer, ua).await
    });

    let mp4_tmp_filepath = tmp_dir.path().join("tmp.mp4");
    let f = tokio::fs::File::create(&mp4_tmp_filepath).await?;
    let writer = tokio::io::BufWriter::new(f);
    let ua = user_agent.clone();
    let url = base_url.clone();
    let mp4_handle = tokio::spawn(async move{
        video.get(url, writer, ua).await
    });

    mp3_handle.await??;
    mp4_handle.await??;

    log::trace!("ffmpeg -i 'mp3_tmp_filepath' -i 'mp4_tmp_filepath' -acodec copy -vcodec copy 'save_file_path'");
    let output = std::process::Command::new("ffmpeg")
        .args(&[
            "-i",
            mp3_tmp_filepath.to_str().unwrap(),
            "-i",
            mp4_tmp_filepath.to_str().unwrap(),
            "-acodec",
            "copy",
            "-vcodec",
            "copy",
            save_file_path.as_ref().to_str().unwrap()
        ])
        .output()?;
    
    tmp_dir.close()?;

    Ok(())
}

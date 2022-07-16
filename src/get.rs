
use std::{path::Path};

#[cfg(feature = "progressbar")]
use tokio::sync::mpsc::{self, Sender};

use easy_scraper::Pattern;
use regex::Regex;
use reqwest::{IntoUrl, Url, header::HeaderValue};
use async_trait::async_trait;
use tokio::io::{AsyncWrite, AsyncWriteExt, BufWriter};
use tokio::fs::File;

#[cfg(feature = "progressbar")]
use indicatif::ProgressBar;

use crate::{audio::Audio, content::Content, error::VimeoError, segment::Segment, video::Video};

#[async_trait]
pub trait Get: Sized {
    fn init_segment(&self) -> &str;
    fn segments(&self) -> &[Segment];
    fn url(&self, base_url: &Url) -> Result<Url, VimeoError>;

    async fn writer<P: AsRef<Path> + Send>(&self, file_path: P) -> Result<BufWriter<File>, VimeoError> {
        let f = File::create(file_path).await?;
        let mut writer = BufWriter::new(f);

        let init_segment = base64::decode(self.init_segment())?;
        writer.write_all(&init_segment).await?;

        Ok(writer)
    }

    async fn write_segments<W, V>(&self, base_url: Url, mut writer: W, user_agent: V) -> Result<(), VimeoError>
    where
        W: AsyncWrite + Unpin + Send,
        V: TryInto<HeaderValue> + Clone + Send,
        V::Error: Into<http::Error>,
    {
        let segments = self.segments();
        for seg in segments {
            let url = base_url.join(&seg.url)?;
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

    #[cfg(feature = "progressbar")]
    async fn write_segments_with_counter<W, V>(self, base_url: Url, mut writer: W, user_agent: V, sender: Sender<()>) -> Result<(), VimeoError>
    where
        W: AsyncWrite + Unpin + Send,
        V: TryInto<HeaderValue> + Clone + Send,
        V::Error: Into<http::Error>,
    {
        let segments = self.segments();
        for seg in segments {
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

            // countup
            sender.send(()).await.unwrap();
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

// async fn get_audio_and_video<U, V>(url: U, user_agent: V) -> Result<(Audio, Video), VimeoError>
// where
//     U: IntoUrl,
//     V: TryInto<HeaderValue>,
//     V::Error: Into<http::Error>,
// {
//     let client = reqwest::Client::builder()
//         .user_agent(user_agent)
//         .build()?;
//     let content = client.get(url)
//         .send()
//         .await?
//         .json::<Content>()
//         .await?;
    
//     content.audio_and_video()
// }

async fn get_contents<U, V>(url: U, user_agent: V) -> Result<Content, VimeoError>
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

    Ok(content)
}

// #[cfg(feature="progressbar")]
// #[allow(dead_code)]
// pub async fn get_audio_with<U1, U2, P, V>(at: U1, from: U2, save_file_path: P, user_agent: V, pb: ProgressBar, downloading_msg: Option<String>, finished_msg: Option<String>) -> Result<(), VimeoError>
// where
//     U1: IntoUrl,
//     U1: IntoUrl,
//     HeaderValue: TryFrom<U2>,
//     <HeaderValue as TryFrom<U2>>::Error: Into<http::Error>,
//     P: AsRef<Path>,
//     V: TryInto<HeaderValue> + Clone + Send + 'static,
//     V::Error: Into<http::Error>,
// {
//     let (info_url, base_url) = get_urls(at, from, user_agent.clone()).await?;
//     log::debug!("info_url: {}", &info_url);
//     log::debug!("base_url: {}", &base_url);

//     let (audio, video) = get_audio_and_video(info_url, user_agent.clone()).await?;
//     log::debug!("audio: {:?}", &audio);
//     log::debug!("video: {:?}", &video);

//     let tmp_dir = tempfile::tempdir()?;
//     log::debug!("tmp_dir: {:?}", &tmp_dir);

//     let total_size = audio.segments().len() + 1;
//     pb.set_length(total_size as u64);

//     let (tx, mut rx) = mpsc::channel(total_size);

//     let mp3_sender = tx.clone();
//     let mp3_user_agent = user_agent.clone();
//     let mp3_tmp_filepath = tmp_dir.path().join("tmp.mp3");
//     let filepath = mp3_tmp_filepath.clone();
//     let url = base_url.clone();
//     let mp3_handle = tokio::spawn(async move {
//         let mp3_base_url = audio.url(&url)?;
//         let mp3_writer = audio.writer(filepath).await?;
//         audio.write_segments_with_counter(mp3_base_url, mp3_writer, mp3_user_agent, mp3_sender).await?;

//         Result::<_, VimeoError>::Ok(())
//     });

//     let downloading_msg = downloading_msg.unwrap_or("downloading".to_string());
//     pb.set_message(downloading_msg);

//     for _ in 0..(total_size - 1) {
//         rx.recv().await.unwrap();
//         pb.inc(1);
//     }

//     mp3_handle.await??;

//     // move
//     tokio::fs::rename(mp3_tmp_filepath, save_file_path).await?;
    
//     tmp_dir.close()?;
    
//     pb.inc(1);
//     let finished_msg = finished_msg.unwrap_or("finished".to_string());
//     pb.finish_with_message(finished_msg);

//     Ok(())
// }

// #[cfg(feature="progressbar")]
// #[allow(dead_code)]
// pub async fn get_video_with<U1, U2, P, V>(at: U1, from: U2, save_file_path: P, user_agent: V, pb: ProgressBar, downloading_msg: Option<String>, finished_msg: Option<String>) -> Result<(), VimeoError>
// where
//     U1: IntoUrl,
//     U1: IntoUrl,
//     HeaderValue: TryFrom<U2>,
//     <HeaderValue as TryFrom<U2>>::Error: Into<http::Error>,
//     P: AsRef<Path>,
//     V: TryInto<HeaderValue> + Clone + Send + 'static,
//     V::Error: Into<http::Error>,
// {
//     let (info_url, base_url) = get_urls(at, from, user_agent.clone()).await?;
//     log::debug!("info_url: {}", &info_url);
//     log::debug!("base_url: {}", &base_url);

//     let (audio, video) = get_audio_and_video(info_url, user_agent.clone()).await?;
//     log::debug!("audio: {:?}", &audio);
//     log::debug!("video: {:?}", &video);

//     let tmp_dir = tempfile::tempdir()?;
//     log::debug!("tmp_dir: {:?}", &tmp_dir);

//     let total_size = video.segments().len() + 1;
//     pb.set_length(total_size as u64);

//     let (tx, mut rx) = mpsc::channel(total_size);

//     let mp4_sender = tx.clone();
//     let mp4_user_agent = user_agent.clone();
//     let mp4_tmp_filepath = tmp_dir.path().join("tmp.mp4");
//     let filepath = mp4_tmp_filepath.clone();
//     let url = base_url.clone();
//     let mp4_handle = tokio::spawn(async move {
//         let mp4_base_url = video.url(&url)?;
//         let mp4_writer = video.writer(filepath).await?;
//         video.write_segments_with_counter(mp4_base_url, mp4_writer, mp4_user_agent, mp4_sender).await?;

//         Result::<_, VimeoError>::Ok(())
//     });

//     let downloading_msg = downloading_msg.unwrap_or("downloading".to_string());
//     pb.set_message(downloading_msg);

//     for _ in 0..(total_size - 1) {
//         rx.recv().await.unwrap();
//         pb.inc(1);
//     }

//     mp4_handle.await??;

//     tokio::fs::rename(mp4_tmp_filepath, save_file_path).await?;
    
//     tmp_dir.close()?;
    
//     pb.inc(1);
//     let finished_msg = finished_msg.unwrap_or("finished".to_string());
//     pb.finish_with_message(finished_msg);

//     Ok(())
// }

// #[cfg(feature="progressbar")]
// #[allow(dead_code)]
// pub async fn get_movie_with<U1, U2, P, V>(at: U1, from: U2, save_file_path: P, user_agent: V, pb: ProgressBar, downloading_msg: Option<String>, finished_msg: Option<String>) -> Result<(), VimeoError>
// where
//     U1: IntoUrl,
//     U1: IntoUrl,
//     HeaderValue: TryFrom<U2>,
//     <HeaderValue as TryFrom<U2>>::Error: Into<http::Error>,
//     P: AsRef<Path>,
//     V: TryInto<HeaderValue> + Clone + Send + 'static,
//     V::Error: Into<http::Error>,
// {
//     let (info_url, base_url) = get_urls(at, from, user_agent.clone()).await?;
//     log::debug!("info_url: {}", &info_url);
//     log::debug!("base_url: {}", &base_url);

//     let (audio, video) = get_audio_and_video(info_url, user_agent.clone()).await?;
//     log::debug!("audio: {:?}", &audio);
//     log::debug!("video: {:?}", &video);

//     let tmp_dir = tempfile::tempdir()?;
//     log::debug!("tmp_dir: {:?}", &tmp_dir);

//     // audio + video + merge
//     let audio_size = audio.segments().len();
//     let video_size = video.segments().len();
//     let total_size = audio_size + video_size + 1;
    
//     pb.set_length(total_size as u64);

//     let (tx, mut rx) = mpsc::channel(total_size);

//     let mp3_sender = tx.clone();
//     let mp3_user_agent = user_agent.clone();
//     let mp3_tmp_filepath = tmp_dir.path().join("tmp.mp3");
//     let filepath = mp3_tmp_filepath.clone();
//     let url = base_url.clone();
//     let mp3_handle = tokio::spawn(async move {
//         let mp3_base_url = audio.url(&url)?;
//         let mp3_writer = audio.writer(filepath).await?;
//         audio.write_segments_with_counter(mp3_base_url, mp3_writer, mp3_user_agent, mp3_sender).await?;

//         Result::<_, VimeoError>::Ok(())
//     });

//     let mp4_sender = tx.clone();
//     let mp4_user_agent = user_agent.clone();
//     let mp4_tmp_filepath = tmp_dir.path().join("tmp.mp4");
//     let filepath = mp4_tmp_filepath.clone();
//     let url = base_url.clone();
//     let mp4_handle = tokio::spawn(async move {
//         let mp4_base_url = video.url(&url)?;
//         let mp4_writer = video.writer(filepath).await?;
//         video.write_segments_with_counter(mp4_base_url, mp4_writer, mp4_user_agent, mp4_sender).await?;

//         Result::<_, VimeoError>::Ok(())
//     });

//     let downloading_msg = downloading_msg.unwrap_or("downloading".to_string());
//     pb.set_message(downloading_msg);

//     for _ in 0..(total_size - 1) {
//         rx.recv().await.unwrap();
//         pb.inc(1);
//     }

//     mp3_handle.await??;
//     mp4_handle.await??;

//     log::trace!("ffmpeg -i 'mp3_tmp_filepath' -i 'mp4_tmp_filepath' -acodec copy -vcodec copy 'save_file_path'");
//     let _ = std::process::Command::new("ffmpeg")
//         .args(&[
//             "-i",
//             mp3_tmp_filepath.to_str().unwrap(),
//             "-i",
//             mp4_tmp_filepath.to_str().unwrap(),
//             "-acodec",
//             "copy",
//             "-vcodec",
//             "copy",
//             save_file_path.as_ref().to_str().unwrap()
//         ])
//         .output()?;
    
//     tmp_dir.close()?;
    
//     pb.inc(1);
//     let finished_msg = finished_msg.unwrap_or("finished".to_string());
//     pb.finish_with_message(finished_msg);

//     Ok(())
// }

// #[allow(dead_code)]
// pub async fn get_audio<U1, U2, P, V>(at: U1, from: U2, save_file_path: P, user_agent: V) -> Result<(), VimeoError>
// where
//     U1: IntoUrl,
//     U1: IntoUrl,
//     HeaderValue: TryFrom<U2>,
//     <HeaderValue as TryFrom<U2>>::Error: Into<http::Error>,
//     P: AsRef<Path>,
//     V: TryInto<HeaderValue> + Clone + Send + 'static,
//     V::Error: Into<http::Error>,
// {
//     let (info_url, base_url) = get_urls(at, from, user_agent.clone()).await?;
//     log::debug!("info_url: {}", &info_url);
//     log::debug!("base_url: {}", &base_url);

//     let (audio, video) = get_audio_and_video(info_url, user_agent.clone()).await?;
//     log::debug!("audio: {:?}", &audio);
//     log::debug!("video: {:?}", &video);

//     let tmp_dir = tempfile::tempdir()?;
//     log::debug!("tmp_dir: {:?}", &tmp_dir);

//     let mp3_user_agent = user_agent.clone();
//     let mp3_tmp_filepath = tmp_dir.path().join("tmp.mp3");
//     let filepath = mp3_tmp_filepath.clone();
//     let url = base_url.clone();
//     let mp3_handle = tokio::spawn(async move {
//         let mp3_base_url = audio.url(&url)?;
//         let mp3_writer = audio.writer(filepath).await?;
//         audio.write_segments(mp3_base_url, mp3_writer, mp3_user_agent).await?;

//         Result::<_, VimeoError>::Ok(())
//     });

//     mp3_handle.await??;

//     // rename
//     tokio::fs::rename(mp3_tmp_filepath, save_file_path).await?;
    
//     tmp_dir.close()?;

//     Ok(())
// }

// #[allow(dead_code)]
// pub async fn get_video<U1, U2, P, V>(at: U1, from: U2, save_file_path: P, user_agent: V) -> Result<(), VimeoError>
// where
//     U1: IntoUrl,
//     U1: IntoUrl,
//     HeaderValue: TryFrom<U2>,
//     <HeaderValue as TryFrom<U2>>::Error: Into<http::Error>,
//     P: AsRef<Path>,
//     V: TryInto<HeaderValue> + Clone + Send + 'static,
//     V::Error: Into<http::Error>,
// {
//     let (info_url, base_url) = get_urls(at, from, user_agent.clone()).await?;
//     log::debug!("info_url: {}", &info_url);
//     log::debug!("base_url: {}", &base_url);

//     let (audio, video) = get_audio_and_video(info_url, user_agent.clone()).await?;
//     log::debug!("audio: {:?}", &audio);
//     log::debug!("video: {:?}", &video);

//     let tmp_dir = tempfile::tempdir()?;
//     log::debug!("tmp_dir: {:?}", &tmp_dir);

//     let mp4_user_agent = user_agent.clone();
//     let mp4_tmp_filepath = tmp_dir.path().join("tmp.mp4");
//     let filepath = mp4_tmp_filepath.clone();
//     let url = base_url.clone();
//     let mp4_handle = tokio::spawn(async move {
//         let mp4_base_url = video.url(&url)?;
//         let mp4_writer = video.writer(filepath).await?;
//         video.write_segments(mp4_base_url, mp4_writer, mp4_user_agent).await?;

//         Result::<_, VimeoError>::Ok(())
//     });

//     mp4_handle.await??;

//     // rename
//     tokio::fs::rename(mp4_tmp_filepath, save_file_path).await?;
    
//     tmp_dir.close()?;

//     Ok(())
// }

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

    let mut content = get_contents(info_url.clone(), user_agent.clone()).await?;
    let audio = content.audios.remove(0);
    let video = content.videos.remove(0);
    let content_base_url = content.base_url.clone();

    let tmp_dir = tempfile::tempdir()?;
    log::debug!("tmp_dir: {:?}", &tmp_dir);

    let audio_tmp_filepath = tmp_dir.path().join("tmp.mp3");
    let audio_base_url = info_url.clone().join(&content_base_url)?.join(&audio.base_url)?;
    let audio_writer = audio.writer(audio_tmp_filepath.clone()).await?;
    let audio_user_agent = user_agent.clone();
    let audio_handle = tokio::spawn(async move {
        let _ = audio.write_segments(audio_base_url, audio_writer, audio_user_agent).await?;
        Result::<_, VimeoError>::Ok(())
    });

    let video_tmp_filepath = tmp_dir.path().join("tmp.mp4");
    let video_base_url = info_url.join(&content_base_url)?.join(&&video.base_url)?;
    let video_writer = video.writer(video_tmp_filepath.clone()).await?;
    let video_user_agent = user_agent.clone();
    let video_handle = tokio::spawn(async move {
        let _ = video.write_segments(video_base_url, video_writer, video_user_agent).await?;
        Result::<_, VimeoError>::Ok(())
    });

    audio_handle.await??;
    video_handle.await??;

    log::trace!("ffmpeg -i 'mp3_tmp_filepath' -i 'mp4_tmp_filepath' -acodec copy -vcodec copy 'save_file_path'");
    let _ = std::process::Command::new("ffmpeg")
        .args(&[
            "-i",
            audio_tmp_filepath.to_str().unwrap(),
            "-i",
            video_tmp_filepath.to_str().unwrap(),
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

// #[allow(dead_code)]
// pub async fn get_movie<U1, U2, P, V>(at: U1, from: U2, save_file_path: P, user_agent: V) -> Result<(), VimeoError>
// where
//     U1: IntoUrl,
//     U1: IntoUrl,
//     HeaderValue: TryFrom<U2>,
//     <HeaderValue as TryFrom<U2>>::Error: Into<http::Error>,
//     P: AsRef<Path>,
//     V: TryInto<HeaderValue> + Clone + Send + 'static,
//     V::Error: Into<http::Error>,
// {
//     let (info_url, base_url) = get_urls(at, from, user_agent.clone()).await?;
//     log::debug!("info_url: {}", &info_url);
//     log::debug!("base_url: {}", &base_url);

//     let (audio, video) = get_audio_and_video(info_url, user_agent.clone()).await?;
//     log::debug!("audio: {:?}", &audio);
//     log::debug!("video: {:?}", &video);

//     let tmp_dir = tempfile::tempdir()?;
//     log::debug!("tmp_dir: {:?}", &tmp_dir);

//     let mp3_user_agent = user_agent.clone();
//     let mp3_tmp_filepath = tmp_dir.path().join("tmp.mp3");
//     let filepath = mp3_tmp_filepath.clone();
//     let url = base_url.clone();
//     let mp3_handle = tokio::spawn(async move {
//         let mp3_base_url = audio.url(&url)?;
//         let mp3_writer = audio.writer(filepath).await?;
//         audio.write_segments(mp3_base_url, mp3_writer, mp3_user_agent).await?;

//         Result::<_, VimeoError>::Ok(())
//     });

//     let mp4_user_agent = user_agent.clone();
//     let mp4_tmp_filepath = tmp_dir.path().join("tmp.mp4");
//     let filepath = mp4_tmp_filepath.clone();
//     let url = base_url.clone();
//     let mp4_handle = tokio::spawn(async move {
//         let mp4_base_url = video.url(&url)?;
//         let mp4_writer = video.writer(filepath).await?;
//         video.write_segments(mp4_base_url, mp4_writer, mp4_user_agent).await?;

//         Result::<_, VimeoError>::Ok(())
//     });

//     mp3_handle.await??;
//     mp4_handle.await??;

//     log::trace!("ffmpeg -i 'mp3_tmp_filepath' -i 'mp4_tmp_filepath' -acodec copy -vcodec copy 'save_file_path'");
//     let _ = std::process::Command::new("ffmpeg")
//         .args(&[
//             "-i",
//             mp3_tmp_filepath.to_str().unwrap(),
//             "-i",
//             mp4_tmp_filepath.to_str().unwrap(),
//             "-acodec",
//             "copy",
//             "-vcodec",
//             "copy",
//             save_file_path.as_ref().to_str().unwrap()
//         ])
//         .output()?;
    
//     tmp_dir.close()?;

//     Ok(())
// }
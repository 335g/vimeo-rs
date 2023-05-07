use std::path::{Path, PathBuf};

use reqwest::Client;
use tempfile::TempDir;
use tokio::sync::OnceCell;
#[cfg(feature = "progressbar")]
use tokio::sync::mpsc::{self, Sender};

use easy_scraper::Pattern;
use regex::Regex;
use reqwest::{IntoUrl, Url, header::HeaderValue};
use crate::config::PlayerConfig;
use crate::content::{write_segments, Contents};

#[cfg(feature = "progressbar")]
use indicatif::ProgressBar;

use crate::{AudioInfo, VideoInfo};
use crate::{content::ContentInfo, error::VimeoError};

async fn get_player_config_regex() -> &'static Regex {
    PLAYER_CONFIG_REGEX.get_or_init(|| async {
        Regex::new(r#"window\.playerConfig = (\{.+\})"#).unwrap()
    }).await
}

static PLAYER_CONFIG_REGEX: OnceCell<Regex> = OnceCell::const_new();

async fn info_url_request<U1, U2>(client: &Client, target: U1, referer: U2) -> Result<Url, VimeoError>
where
    U1: IntoUrl,
    HeaderValue: TryFrom<U2>,
    <HeaderValue as TryFrom<U2>>::Error: Into<http::Error>,
{
    let req = client.get(target)
        .header("referer", referer)
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
    let url = Url::parse(&format!("{}{}{}", &cap[1], &cap[2], &cap[3]))?;
    Ok(url)
}

async fn get_player_config<U1, U2>(client: &Client, target: U1, referer: U2) -> Result<PlayerConfig, VimeoError>
where
    U1: IntoUrl,
    HeaderValue: TryFrom<U2>,
    <HeaderValue as TryFrom<U2>>::Error: Into<http::Error>,
{
    let req = client.get(target)
        .header("referer", referer)
        .build()?;
    let resp = client.execute(req)
        .await?
        .text()
        .await?;
    let pat = Pattern::new(r#"
        <body><script>{{content}}</script></body>
    "#).unwrap();
    
    let matches = pat.matches(&resp);
    let matches1 = matches.first()
        .ok_or(VimeoError::NoPlayerConfig)?;
    let content = matches1.get("content").expect("is script");
    let caps = get_player_config_regex().await
        .captures(content.as_str())
        .ok_or(VimeoError::NoPlayerConfig)?;
    let config = serde_json::from_str(caps.get(1).expect("valid regex").as_str())?;
    
    Ok(config)
}

#[allow(dead_code)]
pub async fn get_movie<U1, U2, P>(client: &Client, target: U1, referer: U2, save_file_path: P) -> Result<(), VimeoError>
where
    U1: IntoUrl,
    U1: IntoUrl,
    HeaderValue: TryFrom<U2>,
    <HeaderValue as TryFrom<U2>>::Error: Into<http::Error>,
    P: AsRef<Path>,
{
    let info_url = info_url_request(client, target, referer).await?;
    println!("{}", info_url.to_string());
    
    let mut content = client.get(info_url.clone()).send().await?.json::<ContentInfo>().await?;
    let audio = content.audio_infos.remove(0);
    let video = content.video_infos.remove(0);
    let content_base_url = content.base_url.clone();

    let tmp_dir = tempfile::tempdir()?;
    log::debug!("tmp_dir: {:?}", &tmp_dir);

    let audio_tmp_filepath = tmp_dir.path().join("tmp.mp3");
    let audio_tmp_filepath2 = audio_tmp_filepath.clone();
    let audio_base_url = info_url.clone().join(&content_base_url)?.join(&audio.base_url)?;
    let audio_client = client.clone();
    let audio_handle = tokio::spawn(async move {
        let _ = write_segments(&audio, &audio_client, audio_base_url, audio_tmp_filepath2, None).await?;
        Result::<_, VimeoError>::Ok(())
    });

    let video_tmp_filepath = tmp_dir.path().join("tmp.mp4");
    let video_tmp_filepath2 = video_tmp_filepath.clone();
    let video_base_url = info_url.join(&content_base_url)?.join(&&video.base_url)?;
    let video_client = client.clone();
    let video_handle = tokio::spawn(async move {
        let _ = write_segments(&video, &video_client, video_base_url, video_tmp_filepath2, None).await?;
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

#[cfg(feature="progressbar")]
#[allow(dead_code)]
pub async fn get_movie_with<U1, U2, P>(client: &Client, target: U1, referer: U2, save_file_path: P, pb: ProgressBar, downloading_msg: Option<String>, finished_msg: Option<String>) -> Result<(), VimeoError>
where
    U1: IntoUrl,
    U1: IntoUrl,
    HeaderValue: TryFrom<U2>,
    <HeaderValue as TryFrom<U2>>::Error: Into<http::Error>,
    P: AsRef<Path>,
{
    let info_url = info_url_request(client, target, referer).await?;
    
    let mut content = client.get(info_url.clone()).send().await?.json::<ContentInfo>().await?;
    let audio = content.audio_infos.remove(0);
    let video = content.video_infos.remove(0);
    let content_base_url = content.base_url.clone();

    let tmp_dir = tempfile::tempdir()?;
    log::debug!("tmp_dir: {:?}", &tmp_dir);

    // audio + video + merge
    let audio_size = audio.segments().len();
    let video_size = video.segments().len();
    let total_size = audio_size + video_size + 1;
    
    pb.set_length(total_size as u64);

    let (tx, mut rx) = mpsc::channel(total_size);

    let audio_sender = tx.clone();
    let audio_callback = || {
        Box::pin(audio_sender.send(()))
    };
    let audio_tmp_filepath = tmp_dir.path().join("tmp.mp3");
    let audio_tmp_filepath2 = audio_tmp_filepath.clone();
    let audio_base_url = info_url.clone().join(&content_base_url)?.join(&audio.base_url)?;
    let audio_client = client.clone();
    let audio_handle = tokio::spawn(async move {
        let _ = write_segments(&audio, &audio_client, audio_base_url, audio_tmp_filepath2, Some(audio_sender)).await?;
        Result::<_, VimeoError>::Ok(())
    });

    let video_sender = tx.clone();
    let video_tmp_filepath = tmp_dir.path().join("tmp.mp4");
    let video_tmp_filepath2 = video_tmp_filepath.clone();
    let video_base_url = info_url.join(&content_base_url)?.join(&&video.base_url)?;
    let video_client = client.clone();
    let video_handle = tokio::spawn(async move {
        let _ = write_segments(&video, &video_client, video_base_url, video_tmp_filepath2, Some(video_sender)).await?;
        Result::<_, VimeoError>::Ok(())
    });

    let downloading_msg = downloading_msg.unwrap_or("downloading".to_string());
    pb.set_message(downloading_msg);

    for _ in 0..(total_size - 1) {
        rx.recv().await.unwrap();
        pb.inc(1);
    }

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
    
    pb.inc(1);
    let finished_msg = finished_msg.unwrap_or("finished".to_string());
    pb.finish_with_message(finished_msg);

    Ok(())
}
use std::{path::PathBuf, io::Write};

use reqwest::Client;
use vimeo_rs::{PlayerConfig, Content};
use clap::Parser;
use url::Url;

#[derive(Debug, Parser)]
struct Args {
    // iframe src
    // ex. https://player.vimeo.com/video/848863798?h=df103ee095
    #[arg(short, long)]
    url: Url,

    // referer
    // ex. https://vimeo.com/848863798
    #[arg(short, long)]
    referer: Url,

    #[arg(short, long)]
    extract_to: Option<PathBuf>,
}

async fn handle(args: Args) -> Result<(), Box<dyn std::error::Error>> {
    let url = args.url.clone();
    let referer = args.referer.as_str();
    let user_agent = ua_generator::ua::spoof_ua();
    let client = Client::builder()
        .user_agent(user_agent)
        .cookie_store(true)
        .build()
        .unwrap();
    let request = client.get(url)
        .header("referer", referer)
        .build()
        .unwrap();
    let html = client.execute(request)
        .await
        .expect("got response")
        .text()
        .await
        .expect("is utf8");
    
    let config = PlayerConfig::try_from(html.as_str())?;
    let summary = &config.video;
    println!("title: {}", summary.title);
    println!("width: {}, height: {}", summary.width, summary.height);
    println!("duration: {}", summary.duration);
    println!("hd: {}, allow_hd: {}, default_to_hd: {}", summary.hd, summary.allow_hd, summary.default_to_hd);
    println!("lang: {}", summary.lang.clone().unwrap_or_default());
    println!("channel: {}", summary.channel_layout);
    
    if let Some(extract_filepath) = args.extract_to {
        if let Some(cdn) = config.dash_default_cdn() {
            let request = client.get(cdn.url.clone())
                .header("referer", referer)
                .build()?;
            let content = client.execute(request)
                .await?
                .json::<Content>()
                // .json::<serde_json::Value>()
                .await?;

            if let Some(audio) = content.mp3_audios().first() {
                println!("bitrate: {}", audio.bitrate);
                println!("channels: {}", audio.channels);
                println!("codecs: {}", audio.codecs);
                println!("duration: {}", audio.duration);
                println!("sample rate: {}", audio.sample_rate);
                
                let buf = content.extract_audio(audio, cdn.url.clone()).await?;

                let mut f = std::fs::File::create(extract_filepath)?;
                f.write_all(&buf).expect("save success");
                println!(" == FIN ==");
            }
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    match handle(args).await {
        Ok(_) => {}
        Err(e) => {
            eprintln!("{:?}", e);
            std::process::exit(1);
        }
    }
}
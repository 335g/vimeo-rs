use std::time::Duration;

use console::Style;
use dialoguer::{theme::ColorfulTheme, FuzzySelect};
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use vimeo_rs::{PlayerConfig, Content};
use clap::Parser;
use url::Url;

#[derive(Debug, Parser)]
struct Args {
    // iframe src
    // ex. "https://player.vimeo.com/video/848863798?h=df103ee095"
    #[arg(short, long)]
    url: Url,

    // referer
    // ex. "https://vimeo.com/848863798"
    #[arg(short, long)]
    referer: Url,
}

fn countup(pb: ProgressBar, step1_duration: Duration, max_len: u64) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        for _ in 0..max_len {
            if pb.is_finished() {
                break
            }

            tokio::time::sleep(step1_duration).await;
            pb.inc(1);
        }
    })
}

async fn handle(args: Args) -> Result<(), Box<dyn std::error::Error>> {
    let pb_len = 100000;
    let pb = ProgressBar::new(pb_len);
    pb.set_style(ProgressStyle::default_spinner());
    pb.set_message("checking ...");
    let pb_handler = countup(pb.clone(), Duration::from_millis(500), pb_len);

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

    pb.set_message("got response");
    
    let config = PlayerConfig::try_from(html.as_str())?;
    let summary = &config.video;

    pb.finish_and_clear();
    pb_handler.await?;

    println!("  ----------------");
    println!("  title: {}", summary.title);
    println!("  width: {}, height: {}", summary.width, summary.height);
    println!("  duration: {}", summary.duration);
    println!("  hd: {}, allow_hd: {}, default_to_hd: {}", summary.hd, summary.allow_hd, summary.default_to_hd);
    println!("  lang: {}", summary.lang.clone().unwrap_or_default());
    println!("  channel: {}", summary.channel_layout);
    println!("  ----------------");

    let theme = ColorfulTheme {
        values_style: Style::new().yellow().dim(),
        ..ColorfulTheme::default()
    };
    let cdns = config.dash_cdns().keys().collect::<Vec<_>>();
    let cdn_index = FuzzySelect::with_theme(&theme)
        .with_prompt("Which CDN to check?")
        .default(0)
        .items(&cdns)
        .interact()?;
    let cdn_key = cdns[cdn_index];
    let cdn = config.dash_cdns().get(cdn_key).expect("is CDN");
    
    let pb = ProgressBar::new(pb_len);
    pb.set_style(ProgressStyle::default_spinner());
    pb.set_message("checking CDN ...");
    let pb_handle = countup(pb.clone(), Duration::from_millis(500), pb_len);

    let request = client.get(cdn.url.clone())
        .header("referer", referer)
        .build()?;
    let content = client.execute(request)
        .await?
        .json::<Content>()
        .await?;

    pb.finish_and_clear();
    pb_handle.await?;

    let audios = content.audio_exps()
        .iter()
        .map(|audio| format!("codecs: {}", audio.codecs))
        .collect::<Vec<_>>();

    let audio_index = FuzzySelect::with_theme(&theme)
        .with_prompt("Which audio do you select?")
        .default(0)
        .items(&audios)
        .interact()?;

    let audio_exps = content.audio_exps();
    let audio_exp = audio_exps.get(audio_index).expect("correct index");
    let audio = content.eq_audio(audio_exp).expect("is same audio");

    println!(" ");
    println!("    codecs: {}", audio.codecs);
    println!("   bitrate: {}", audio.bitrate);
    println!("  channels: {}", audio.channels);
    println!("  duration: {}", audio.duration);
    println!(" ");
    
    println!(" ** That's all! **");
    println!(" ** If you want to check the specific contents, use `Audio.extract`. **");

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
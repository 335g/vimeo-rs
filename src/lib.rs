
mod error;
mod player;
mod content;

use scraper::{Html, Selector};
use tokio::sync::OnceCell;
use regex::Regex;

pub use player::PlayerConfig;
pub use content::{Content, Audio, Video};
pub use error::VimeoError;

static CONFIG_REGEX: OnceCell<Regex> = OnceCell::const_new();

async fn config_regex() -> &'static Regex {
    CONFIG_REGEX.get_or_init(|| async {
        Regex::new(r#"window\.playerConfig = (?P<obj>[^;]+)</script>"#).unwrap()
    }).await
}

pub async fn player_config(html: &str) -> Result<PlayerConfig, VimeoError> {
    let html = Html::parse_document(html);
    let selector = Selector::parse("script").unwrap();
    let config_regex = config_regex().await;
    let config = html.select(&selector)
        .map(|elem| elem.html())
        .filter(|s| config_regex.is_match(s.trim()))
        .map(|s| {
            let caps = config_regex.captures(s.trim()).expect("match config regex");
            let obj = caps.name("obj").expect("is obj");
            obj.as_str().to_string()
        })
        .next()
        .ok_or(VimeoError::NoPlayerConfig)?;

    let config: PlayerConfig = serde_json::from_str(&config)
        .map_err(|_| VimeoError::InvalidPlayerConfig)?;
    
    Ok(config)
}
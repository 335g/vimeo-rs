use easy_scraper::Pattern;
use http::HeaderValue;
use regex::Regex;
use reqwest::{IntoUrl, Client};
use serde::Deserialize;
use time::OffsetDateTime;
use tokio::sync::OnceCell;
use url::Url;
use std::collections::HashMap;

use crate::error::VimeoError;

async fn get_player_config_regex() -> &'static Regex {
    PLAYER_CONFIG_REGEX.get_or_init(|| async {
        Regex::new(r#"window\.playerConfig = (\{.+\})"#).unwrap()
    }).await
}

static PLAYER_CONFIG_REGEX: OnceCell<Regex> = OnceCell::const_new();

#[allow(dead_code)]
#[readonly::make]
#[derive(Debug, Deserialize)]
pub struct PlayerConfig {
    pub cdn_url: Url,
    pub vimeo_api_url: String,
    pub request: Request,
}

#[allow(dead_code)]
#[readonly::make]
#[derive(Debug, Deserialize)]
pub struct Request {
    pub files: Files,
    pub file_codecs: FileCodecs,
    pub referrer: Url,
    pub cookie_domain: String,
    pub signature: String,
    
    #[serde(deserialize_with = "time::serde::timestamp::deserialize")]
    pub timestamp: OffsetDateTime,
    pub expires: u64,

    pub urls: FileUrls,
}

#[allow(dead_code)]
#[readonly::make]
#[derive(Debug, Deserialize)]
pub struct Files {
    pub dash: Dash,
    pub hls: Hls,
}

#[allow(dead_code)]
#[readonly::make]
#[derive(Debug, Deserialize)]
pub struct FileCodecs {
    pub av1: Vec<CodecId>,
    pub avc: Vec<CodecId>,
    pub hevc: HashMap<String, Vec<CodecId>>,
}

#[allow(dead_code)]
#[readonly::make]
#[derive(Debug, Deserialize)]
pub struct Dash {
    pub cdns: HashMap<CdnIdentifier, Cdn>,
    pub default_cdn: CdnIdentifier,
    pub separate_av: bool,
    pub streams: Vec<Stream>,
    pub streams_avc: Vec<Stream>,
}

#[allow(dead_code)]
#[readonly::make]
#[derive(Debug, Deserialize)]
pub struct Hls {
    pub cdns: HashMap<CdnIdentifier, Cdn>,
    pub default_cdn: CdnIdentifier,
    pub separate_av: bool,
}

trait HasCdn {
    fn default_cdn(&self) -> &CdnIdentifier;
    fn cdns(self) -> HashMap<CdnIdentifier, Cdn>;
}

impl HasCdn for Dash {
    fn default_cdn(&self) -> &CdnIdentifier {
        &self.default_cdn
    }

    fn cdns(self) -> HashMap<CdnIdentifier, Cdn> {
        self.cdns
    }
}

impl HasCdn for Hls {
    fn default_cdn(&self) -> &CdnIdentifier {
        &self.default_cdn
    }

    fn cdns(self) -> HashMap<CdnIdentifier, Cdn> {
        self.cdns
    }
}

#[allow(dead_code)]
#[readonly::make]
#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize)]
pub struct CdnIdentifier(pub String);

#[allow(dead_code)]
#[readonly::make]
#[derive(Debug, Deserialize)]
pub struct Cdn {
    pub url: Url,
    pub avc_url: Url,
    pub origin: String,
}

#[allow(dead_code)]
#[readonly::make]
#[derive(Debug, Deserialize)]
pub struct Stream {
    pub profile: String,
    pub id: CodecId,
    pub fps: f32,
    pub quality: String,
}

#[allow(dead_code)]
#[readonly::make]
#[derive(Debug, PartialEq, Eq, Deserialize)]
pub struct CodecId(pub String);

#[allow(dead_code)]
#[readonly::make]
#[derive(Debug, Deserialize)]
pub struct FileUrls {
    pub js: Url,
    pub locales_js: HashMap<String, Url>,
    pub vuid_js: Url,
}

async fn player_config<U1, U2>(client: &Client, target: U1, referer: U2) -> Result<PlayerConfig, VimeoError>
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

fn priority_cdns<T: HasCdn>(x: T) -> Vec<Cdn> {
    let default_cdn_id = x.default_cdn().clone();
    let mut cdns = x.cdns();
    let default_cdn = cdns.remove(&default_cdn_id);

    let mut cdns = cdns.into_iter()
        .map(|(_, cdn)| cdn)
        .collect::<Vec<_>>();
    
    if let Some(cdn) = default_cdn {
        cdns.insert(0, cdn);
    }

    cdns
}

pub async fn dash_cdns<U1, U2>(client: &Client, target: U1, referer: U2) -> Result<Vec<Cdn>, VimeoError>
where
    U1: IntoUrl,
    HeaderValue: TryFrom<U2>,
    <HeaderValue as TryFrom<U2>>::Error: Into<http::Error>,
{
    let config = player_config(client, target, referer).await?;
    let dash = config.request.files.dash;
    let cdns = priority_cdns(dash);

    Ok(cdns)
}

pub async fn hls_cdns<U1, U2>(client: &Client, target: U1, referer: U2) -> Result<Vec<Cdn>, VimeoError>
where
    U1: IntoUrl,
    HeaderValue: TryFrom<U2>,
    <HeaderValue as TryFrom<U2>>::Error: Into<http::Error>,
{
    let config = player_config(client, target, referer).await?;
    let hls = config.request.files.hls;
    let cdns = priority_cdns(hls);

    Ok(cdns)
}
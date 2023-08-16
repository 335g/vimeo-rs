#![allow(dead_code)]

use std::{collections::HashMap, sync::OnceLock, ops::Deref};
use regex::Regex;
use scraper::{Html, Selector};
use serde::{Deserialize, Deserializer};
use url::Url;
use uuid::Uuid;
use time::{OffsetDateTime, PrimitiveDateTime};
use parse_display::Display;

use crate::error::VimeoError;

static CONFIG_REGEX: OnceLock<Regex> = OnceLock::new();

fn config_regex() -> &'static Regex {
    CONFIG_REGEX.get_or_init(|| {
        Regex::new(r#"window\.playerConfig = (?P<obj>[^;]+)</script>"#).unwrap()
    })
}

#[readonly::make]
#[derive(Debug, Deserialize)]
pub struct PlayerConfig {
    request: Request,
    pub video: Summary,
    pub seo: Seo,
}

impl PlayerConfig {
    #[inline]
    pub fn dash(&self) -> &Dash {
        &self.request.files.dash
    }

    #[inline]
    pub fn dash_cdns(&self) -> &HashMap<String, Cdn> {
        &self.request.files.dash.cdns.0
    }

    pub fn dash_default_cdn(&self) -> &Cdn {
        let default_cdn = self.dash_default_cdn_key();
        let dash = self.dash();

        dash.cdns.get(default_cdn).expect("is default cdn")
    }

    #[inline]
    pub fn dash_default_cdn_key(&self) -> &str {
        &self.request.files.dash.default_cdn
    }

    #[inline]
    pub fn hls(&self) -> &Hls {
        &self.request.files.hls
    }

    #[inline]
    pub fn hls_cdns(&self) -> &HashMap<String, Cdn> {
        &self.request.files.hls.cdns.0
    }

    pub fn hls_default_cdn(&self) -> &Cdn {
        let default_cdn = self.hls_default_cdn_key();
        let hls = self.hls();

        hls.cdns.get(default_cdn).expect("is default cdn")
    }

    #[inline]
    pub fn hls_default_cdn_key(&self) -> &str {
        &self.request.files.hls.default_cdn
    }
}

impl TryFrom<&str> for PlayerConfig {
    type Error = VimeoError;

    fn try_from(html: &str) -> Result<Self, Self::Error> {
        let html = Html::parse_document(html);
        let selector = Selector::parse("script").unwrap();
        let config_regex = config_regex();
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

        let config: PlayerConfig = serde_json::from_str(&config)?;
        
        Ok(config)
    }
}

#[readonly::make]
#[derive(Debug, Deserialize)]
pub struct Request {
    files: Files,
    lang: String,
    referrer: Url,
    cookie_domain: String,
    signature: String,
    #[serde(with = "time::serde::timestamp")]
    pub timestamp: OffsetDateTime,
    pub expires: u64,
    pub file_codecs: FileCodecs,
}

#[derive(Debug, Deserialize)]
pub struct Files {
    dash: Dash,
    hls: Hls,
}

#[derive(Debug, Deserialize)]
pub struct Dash {
    cdns: Cdns,
    default_cdn: String,
    separate_av: bool,
    streams: Vec<Stream>,
    streams_avc: Vec<Stream>,
}

#[derive(Debug, Deserialize)]
pub struct Hls {
    cdns: Cdns,
    default_cdn: String,
    separate_av: bool,
}

#[derive(Debug, Deserialize)]
pub struct Cdns(HashMap<String, Cdn>);

impl Deref for Cdns {
    type Target = HashMap<String, Cdn>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[readonly::make]
#[derive(Debug, Deserialize)]
pub struct Cdn {
    pub avc_url: Url,
    pub origin: String,
    pub url: Url,
}

#[derive(Debug, Deserialize)]
pub struct Stream { 
    profile: String,
    id: Uuid,
    fps: f32,
    quality: Quality,
}

#[derive(Debug, Deserialize)]
pub enum Quality {
    #[serde(rename = "1080p")]
    P1080,
    #[serde(rename = "720p")]
    P720,
    #[serde(rename = "540p")]
    P540,
    #[serde(rename = "360p")]
    P360,
    #[serde(rename = "240p")]
    P240,
}

#[readonly::make]
#[derive(Debug, Deserialize)]
pub struct FileCodecs {
    pub av1: Vec<String>,
    pub avc: Vec<String>,
}

#[readonly::make]
#[derive(Debug, Deserialize)]
pub struct Summary {
    pub id: usize,
    pub title: String,
    pub width: usize,
    pub height: usize,
    pub duration: usize,
    // pub url: Url,
    // pub share_url: Url,
    pub hd: usize,
    pub allow_hd: usize,
    pub default_to_hd: usize,
    pub privacy: Privacy,
    pub lang: Option<String>,
    pub owner: Owner,
    pub channel_layout: ChannelLayout,
}

#[non_exhaustive]
#[derive(Debug, Display, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Privacy {
    Anybody,
    Disable,
}

#[readonly::make]
#[derive(Debug, Deserialize)]
pub struct Owner {
    pub id: usize,
    pub name: String,
    pub img: Url,
    #[serde(rename = "img_2x")]
    pub img2x: Url,
    pub url: Url,
    pub account_type: AccountType,
}

#[non_exhaustive]
#[derive(Debug, Display, Deserialize)]
#[serde(rename_all = "snake_case")]
#[display(style = "snake_case")]
pub enum AccountType {
    Pro,
}

#[non_exhaustive]
#[derive(Debug, Display, Deserialize)]
#[serde(rename_all = "snake_case")]
#[display(style = "snake_case")]
pub enum ChannelLayout {
    Stereo
}

#[readonly::make]
#[derive(Debug, Deserialize)]
pub struct Seo {
    description: String,
    #[serde(deserialize_with = "deserialize_upload_date")]
    upload_date: PrimitiveDateTime,
    embed_url: Url,
    thumbnail: Url,
    canonical_url: Url,
}

fn deserialize_upload_date<'de, D>(deserializer: D) -> Result<PrimitiveDateTime, D::Error>
where
    D: Deserializer<'de>
{
    let s = String::deserialize(deserializer)?;
    let format = time::format_description::parse("[year]-[month]-[day] [hour]:[minute]:[second]").expect("correct format");
    let date = PrimitiveDateTime::parse(&s, &format)
        .map_err(|e| serde::de::Error::custom(format!("cannot parse to PrimitiveDateTime: {:?}", e)))?;

    Ok(date)
}
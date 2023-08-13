#![allow(dead_code)]

use std::{collections::HashMap, ops::Deref};
use regex::Regex;
use serde::Deserialize;
use tokio::sync::OnceCell;
use url::Url;
use uuid::Uuid;
use time::OffsetDateTime;

use crate::VimeoError;

static MASTER_URL_REGEX: OnceCell<Regex> = OnceCell::const_new();

async fn master_url_regex() -> &'static Regex {
    MASTER_URL_REGEX.get_or_init(|| async {
        Regex::new(r#"(?P<base>https://.+/sep/)video/[0-9a-z]{8}(,[0-9a-z]{8})*/audio/[0-9a-z]{8}(,[0-9a-z]{8})*/master\.json.+"#).unwrap()
    }).await
}

#[derive(Debug, Deserialize)]
pub struct PlayerConfig {
    request: Request,
}

impl PlayerConfig {
    pub fn dash_cdns(&self) -> &HashMap<String, Cdn> {
        &self.request.files.dash.cdns.0
    }

    pub fn dash_default_cdn(&self) -> Option<&Cdn> {
        let default_cdn = &self.request.files.dash.default_cdn;

        self.request.files.dash.cdns.0.get(default_cdn)
    }

    pub fn master_urls(&self) -> Vec<MasterUrl> {
        self.request
            .files
            .dash
            .cdns
            .0
            .values()
            .into_iter()
            .map(|cdn| {
                MasterUrl {
                    avc_url: cdn.avc_url.clone(),
                    url: cdn.url.clone(),
                }
            })
            .collect()
    }

    pub fn hls_cdns(&self) -> &HashMap<String, Cdn> {
        &self.request.files.hls.cdns.0
    }

    pub fn hls_default_cdn(&self) -> Option<&Cdn> {
        let default_cdn = &self.request.files.hls.default_cdn;

        self.request.files.hls.cdns.0.get(default_cdn)
    }
}

#[readonly::make]
#[derive(Debug)]
pub struct MasterUrl {
    pub avc_url: Url,
    pub url: Url,
}

impl MasterUrl {
    pub async fn base_url(&self) -> Result<Url, VimeoError> {
        let caps = master_url_regex().await
            .captures(&self.url.as_str())
            .ok_or(VimeoError::InvalidMasterUrl)?;

        let url = caps.name("base").expect("is base url").as_str();
        let url = Url::parse(url).expect("is url");
        Ok(url)
    }

    pub async fn avc_base_url(&self) -> Result<Url, VimeoError> {
        let caps = master_url_regex().await
            .captures(&self.avc_url.as_str())
            .ok_or(VimeoError::InvalidMasterUrl)?;

        let url = caps.name("base").expect("is base url").as_str();
        let url = Url::parse(url).expect("is url");
        Ok(url)
    }
}

#[derive(Debug, Deserialize)]
pub struct Request {
    files: Files,
    lang: String,
    referrer: Url,
    cookie_domain: String,
    signature: String,
    #[serde(with = "time::serde::timestamp")]
    timestamp: OffsetDateTime,
    expires: u64,
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

#[readonly::make]
#[derive(Debug, Deserialize)]
pub struct Cdn {
    pub avc_url: Url,
    pub origin: String,
    pub url: Url,
}

#[derive(Debug, Deserialize)]
pub struct Stream { 
    profile: Uuid,
    id: Uuid,
    fps: f32,
    quality: Quality,
}

#[derive(Debug, Deserialize)]
pub enum Quality {
    #[serde(rename = "360p")]
    P360,
    #[serde(rename = "240p")]
    P240,
}

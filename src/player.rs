#![allow(dead_code)]

use std::{collections::HashMap, sync::OnceLock};
use regex::Regex;
use serde::Deserialize;
use url::Url;
use uuid::Uuid;
use time::OffsetDateTime;

static MASTER_URL_REGEX: OnceLock<Regex> = OnceLock::new();

fn master_url_regex() -> &'static Regex {
    MASTER_URL_REGEX.get_or_init(|| {
        Regex::new(r#"(?P<base>https://.+/sep/)video/[0-9a-z]{8}(,[0-9a-z]{8})*/audio/[0-9a-z]{8}(,[0-9a-z]{8})*/master\.json.+"#).unwrap()
    })
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

    pub fn hls_cdns(&self) -> &HashMap<String, Cdn> {
        &self.request.files.hls.cdns.0
    }

    pub fn hls_default_cdn(&self) -> Option<&Cdn> {
        let default_cdn = &self.request.files.hls.default_cdn;

        self.request.files.hls.cdns.0.get(default_cdn)
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

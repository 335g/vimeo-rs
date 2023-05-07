use serde::Deserialize;
use time::OffsetDateTime;
use url::Url;
use std::collections::HashMap;

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
    pub cdns: HashMap<Cdn, CdnDetail>,
    pub default_cdn: Cdn,
    pub separate_av: bool,
    pub streams: Vec<Stream>,
    pub streams_avc: Vec<Stream>,
}

#[allow(dead_code)]
#[readonly::make]
#[derive(Debug, Eq, PartialEq, Hash, Deserialize)]
pub struct Cdn(pub String);

#[allow(dead_code)]
#[readonly::make]
#[derive(Debug, Deserialize)]
pub struct CdnDetail {
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
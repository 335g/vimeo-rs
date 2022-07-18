use serde::Deserialize;

#[readonly::make]
#[derive(Debug, Clone, Deserialize)]
pub struct Segment {
    pub start: f64,
    pub end: f64,
    pub url: String,
    pub size: usize,
}

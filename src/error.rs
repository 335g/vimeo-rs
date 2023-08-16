
#[derive(Debug, thiserror::Error)]
pub enum VimeoError {
    #[error("no player.config object")]
    NoPlayerConfig,

    #[error("json ser/de error: {0:?}")]
    Json(#[from] serde_json::Error),

    #[error("time parse error: {0:?}")]
    TimeParse(#[from] time::error::Parse),

    #[error("invalid url: {0:?}")]
    InvalidUrl(String),

    #[error("failed assemble content: {reason}")]
    CannotExtract { reason: String }
}
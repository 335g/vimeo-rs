use thiserror::Error;

#[derive(Debug, Error)]
pub enum VimeoError {
    #[error("IO error: {0:?}")]
    Io(#[from] std::io::Error),

    #[error("base64 decode error: {0:?}")]
    Base64Decode(#[from] base64::DecodeError),

    #[error("Networking error: {0:?}")]
    Reqwest(#[from] reqwest::Error),

    #[error("tokio task join error: {0:?}")]
    Join(#[from] tokio::task::JoinError),

    #[error("invalid json: {0:?}")]
    Json(#[from] serde_json::Error),

    #[error("Cannot find the ffmpeg")]
    CannotFindFFmpeg,

    #[error("No 'master.json' description found in the response")]
    CannotDeserializeThe1stResponse,

    #[error("Parse url error: {0:?}")]
    ParseUrl(#[from] url::ParseError),

    #[error("No Audio")]
    NoAudio,

    #[error("No Video")]
    NoVideo,

    #[error("No Player Config")]
    NoPlayerConfig,

    #[error("is not success: {0:?}")]
    IsNotSuccess(reqwest::StatusCode),
}

#[derive(Debug, thiserror::Error)]
pub enum VimeoError {
    #[error("no player.config object")]
    NoPlayerConfig,

    #[error("invalid player config")]
    InvalidPlayerConfig,

    #[error("invalid url: {0:?}")]
    InvalidUrl(String),

    #[error("failed assemble content: {reason}")]
    FailedAssembleContent { reason: String }
}

#[derive(Debug, thiserror::Error)]
pub enum VimeoError {
    #[error("no player.config object")]
    NoPlayerConfig,

    #[error("invalid player config")]
    InvalidPlayerConfig,

    #[error("invalid url (master.json)")]
    InvalidMasterUrl,
}
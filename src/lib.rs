
mod error;
mod audio;
mod video;
mod content;
mod segment;
mod get;

pub use error::VimeoError;
pub use get::get_movie;
pub use audio::AudioInfo;
pub use video::VideoInfo;
pub use content::ContentInfo;

#[cfg(feature = "progressbar")]
pub use indicatif::{ProgressBar, ProgressStyle, MultiProgress};

#[cfg(feature = "progressbar")]
pub use get::get_movie_with;

pub use reqwest::Client;

mod error;
mod audio;
mod video;
mod content;
mod segment;
mod get;

pub use error::VimeoError;
pub use get::get_movie;
pub use audio::Audio;
pub use video::Video;
pub use content::Content;
pub use get::Get;

#[cfg(feature = "progressbar")]
pub use indicatif::{ProgressBar, ProgressStyle, MultiProgress};

#[cfg(feature = "progressbar")]
pub use get::{get_audio_with, get_video_with, get_movie_with};
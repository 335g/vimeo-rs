use serde::Deserialize;

use crate::{VimeoError, audio::Audio, video::Video};

#[readonly::make]
#[derive(Debug, Deserialize)]
pub struct Content {
    pub clip_id: String,
    pub base_url: String,

    #[serde(rename = "audio")]
    pub audios: Vec<Audio>,

    #[serde(rename = "video")]
    pub videos: Vec<Video>,
}

// impl Content {
//     pub fn audio_and_video(self) -> Result<(Audio, Video), VimeoError> {
//         let audio = self.audio.into_iter()
//             .next()
//             .ok_or(VimeoError::NoAudio)?;

//         let video = self.video
//             .into_iter()
//             .fold(None, |acc, x| {
//                 match acc {
//                     None => Some(x),
//                     Some(v) => {
//                         if v.height() < x.height() {
//                             Some(x)
//                         } else {
//                             Some(v)
//                         }
//                     }
//                 }
//             })
//             .ok_or(VimeoError::NoVideo)?;

//         return Ok((audio, video))
//     }
// }
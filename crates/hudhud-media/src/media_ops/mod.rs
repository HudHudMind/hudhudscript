//! Shared media builtins — image/audio/video info via native + ffprobe/ImageMagick.
//!
//! Single source of truth for VM and interpreter runtimes (Kural 7).

use hudhudscript_bytecode::Value16;
use hudhudscript_errors::{Error, ErrorCode, HudHudResult};

/// Enum identifying each operation for zero-cost dispatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScriptMethodId {
    ImageInfo,
    ImageResize,
    ImageConvert,
    AudioInfo,
    VideoInfo,
    Transcode,
    Thumbnail,
    FileType,
}

impl std::str::FromStr for ScriptMethodId {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "image_info" => Ok(Self::ImageInfo),
            "image_resize" => Ok(Self::ImageResize),
            "image_convert" => Ok(Self::ImageConvert),
            "audio_info" => Ok(Self::AudioInfo),
            "video_info" => Ok(Self::VideoInfo),
            "transcode" => Ok(Self::Transcode),
            "thumbnail" => Ok(Self::Thumbnail),
            "file_type" => Ok(Self::FileType),
            _ => Err(runtime_error(format!("Unknown method: {}", s))),
        }
    }
}

/// Zero-cost enum dispatch.
pub fn dispatch(method: ScriptMethodId, args: &[Value16]) -> HudHudResult<Value16> {
    match method {
        ScriptMethodId::ImageInfo => image::image_info(args),
        ScriptMethodId::ImageResize => image::image_resize(args),
        ScriptMethodId::ImageConvert => image::image_convert(args),
        ScriptMethodId::AudioInfo => audio_video::audio_info(args),
        ScriptMethodId::VideoInfo => audio_video::video_info(args),
        ScriptMethodId::Transcode => audio_video::transcode(args),
        ScriptMethodId::Thumbnail => util::thumbnail(args),
        ScriptMethodId::FileType => util::file_type(args),
    }
}

mod audio_video;
mod image;
mod util;

pub use audio_video::*;
pub use image::*;
pub use util::*;

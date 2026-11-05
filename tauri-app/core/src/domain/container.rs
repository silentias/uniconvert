use serde::{Deserialize, Serialize};

use super::{AudioCodec, VideoCodec};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Container {
    Mp4,
    Mkv,
    Webm,
    Mov,
    Avi,
    Mp3,
    Wav,
    Flac,
    Ogg,
    Jpeg,
    Png,
    Webp,
    Avif,
    Gif,
    Bmp,
    Tiff,
    Ico,
    Aac,
    Opus,
    M4a,
    Wma,
    Aiff,
}

impl Container {
    pub fn from_image_extension(extension: &str) -> Option<Self> {
        // TODO: Позже вынести aliases и доступность форматов в динамический реестр.
        match extension
            .trim_start_matches('.')
            .to_ascii_lowercase()
            .as_str()
        {
            "jpg" | "jpeg" => Some(Self::Jpeg),
            "png" => Some(Self::Png),
            "webp" => Some(Self::Webp),
            "avif" => Some(Self::Avif),
            "gif" => Some(Self::Gif),
            "bmp" => Some(Self::Bmp),
            "tif" | "tiff" => Some(Self::Tiff),
            "ico" => Some(Self::Ico),
            _ => None,
        }
    }

    pub fn extension(self) -> &'static str {
        // TODO: Позже учитывать пользовательский alias расширения (jpeg/jpg, tif/tiff).
        match self {
            Self::Mp4 => "mp4",
            Self::Mkv => "mkv",
            Self::Webm => "webm",
            Self::Mov => "mov",
            Self::Avi => "avi",
            Self::Mp3 => "mp3",
            Self::Wav => "wav",
            Self::Flac => "flac",
            Self::Ogg => "ogg",
            Self::Jpeg => "jpg",
            Self::Png => "png",
            Self::Webp => "webp",
            Self::Avif => "avif",
            Self::Gif => "gif",
            Self::Bmp => "bmp",
            Self::Tiff => "tiff",
            Self::Ico => "ico",
            Self::Aac => "aac",
            Self::Opus => "opus",
            Self::M4a => "m4a",
            Self::Wma => "wma",
            Self::Aiff => "aiff",
        }
    }

    pub fn supports(self, video: Option<VideoCodec>, audio: Option<AudioCodec>) -> bool {
        // TODO: Сначала проверить базовые пары контейнеров и кодеков.
        // TODO: Затем учесть версии контейнеров, профили и ограничения платформ.
        // TODO: Позже формировать объяснение несовместимости для интерфейса.
        todo!("container compatibility matrix is not implemented")
    }
}

use crate::{
    domain::{AudioCodec, ConvertError, MediaInfo, MediaKind, VideoCodec},
    infra::ffprobe,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{path::Path, sync::atomic::AtomicBool};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioInfo {
    pub duration_seconds: Option<f64>,
    pub file_size: u64,
    pub format: Option<String>,
    pub codec: Option<String>,
    pub bitrate: Option<u64>,
    pub sample_rate_hz: Option<u32>,
    pub channels: Option<u32>,
    pub channel_layout: Option<String>,
}

pub fn probe_audio(path: &Path, cancelled: &AtomicBool) -> Result<AudioInfo, ConvertError> {
    probe_audio_stream(path, cancelled, false)
}

pub fn probe_audio_stream(
    path: &Path,
    cancelled: &AtomicBool,
    allow_video: bool,
) -> Result<AudioInfo, ConvertError> {
    let metadata = std::fs::metadata(path)
        .map_err(|error| ConvertError::InvalidInput(format!("{}: {error}", path.display())))?;
    if !metadata.is_file() {
        return Err(ConvertError::InvalidInput(
            "Ожидается локальный аудиофайл".into(),
        ));
    }
    let path = std::fs::canonicalize(path).map_err(|error| ConvertError::Io(error.to_string()))?;
    let json: Value = serde_json::from_str(&ffprobe::probe_json_cancellable(&path, cancelled)?)
        .map_err(|error| ConvertError::ProcessFailed(error.to_string()))?;
    parse_audio_stream_info(&json, metadata.len(), allow_video)
}

#[cfg(test)]
fn parse_audio_info(json: &Value, file_size: u64) -> Result<AudioInfo, ConvertError> {
    parse_audio_stream_info(json, file_size, false)
}

fn parse_audio_stream_info(
    json: &Value,
    file_size: u64,
    allow_video: bool,
) -> Result<AudioInfo, ConvertError> {
    let streams = json["streams"]
        .as_array()
        .ok_or_else(|| ConvertError::InvalidInput("В файле не найдены аудиопотоки".into()))?;
    if !allow_video
        && streams.iter().any(|stream| {
            stream["codec_type"] == "video" && stream["disposition"]["attached_pic"] != 1
        })
    {
        return Err(ConvertError::InvalidInput(
            "Для видео выберите режим извлечения аудио".into(),
        ));
    }
    let audio = streams
        .iter()
        .find(|stream| stream["codec_type"] == "audio")
        .ok_or_else(|| ConvertError::InvalidInput("В файле не найдены аудиопотоки".into()))?;
    let number = |value: &Value| {
        value
            .as_f64()
            .or_else(|| value.as_str()?.parse::<f64>().ok())
    };
    let integer = |value: &Value| {
        value
            .as_u64()
            .or_else(|| value.as_str()?.parse::<u64>().ok())
    };
    let duration_seconds = number(&audio["duration"])
        .or_else(|| number(&json["format"]["duration"]))
        .filter(|value| value.is_finite() && *value > 0.0);
    Ok(AudioInfo {
        duration_seconds,
        file_size,
        format: json["format"]["format_name"].as_str().map(str::to_owned),
        codec: audio["codec_name"].as_str().map(str::to_owned),
        bitrate: integer(&audio["bit_rate"]).or_else(|| integer(&json["format"]["bit_rate"])),
        sample_rate_hz: integer(&audio["sample_rate"]).and_then(|value| value.try_into().ok()),
        channels: integer(&audio["channels"]).and_then(|value| value.try_into().ok()),
        channel_layout: audio["channel_layout"].as_str().map(str::to_owned),
    })
}

#[cfg(test)]
mod audio_tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn accepts_album_art_but_rejects_video_and_silent_files() {
        let mut value = json!({"streams":[
            {"codec_type":"video","disposition":{"attached_pic":1}},
            {"codec_type":"audio","codec_name":"mp3","duration":"12.5","sample_rate":"44100","channels":2}
        ],"format":{"duration":"13","bit_rate":"192000"}});
        let info = parse_audio_info(&value, 123).unwrap();
        assert_eq!(info.duration_seconds, Some(12.5));
        assert_eq!(info.sample_rate_hz, Some(44100));
        assert_eq!(info.channels, Some(2));
        value["streams"][0]["disposition"]["attached_pic"] = json!(0);
        assert!(parse_audio_info(&value, 123).is_err());
        assert!(parse_audio_stream_info(&value, 123, true).is_ok());
        assert!(
            parse_audio_stream_info(&json!({"streams": [{"codec_type": "video"}]}), 0, true)
                .is_err()
        );
        assert!(parse_audio_info(&json!({"streams":[]}), 0).is_err());
    }
}

pub fn probe(path: &Path) -> Result<MediaInfo, ConvertError> {
    // TODO: Затем нормализовать дополнительные потоки и необычные файлы.
    // TODO: Позже добавить кэш по пути и времени изменения.
    let json: Value = serde_json::from_str(&ffprobe::probe_json(path)?)
        .map_err(|error| ConvertError::ProcessFailed(error.to_string()))?;
    let streams = json["streams"].as_array().cloned().unwrap_or_default();
    let video = streams
        .iter()
        .find(|stream| stream["codec_type"] == "video");
    let audio = streams
        .iter()
        .find(|stream| stream["codec_type"] == "audio");
    let format = &json["format"];
    let parse_number = |value: &Value| value.as_str().and_then(|text| text.parse::<f64>().ok());
    let duration_seconds = parse_number(&format["duration"]).or_else(|| {
        streams
            .iter()
            .find_map(|stream| parse_number(&stream["duration"]))
    });
    let bitrate = format["bit_rate"]
        .as_str()
        .and_then(|value| value.parse::<u64>().ok());
    let file_size = format["size"]
        .as_str()
        .and_then(|value| value.parse::<u64>().ok())
        .or_else(|| std::fs::metadata(path).ok().map(|metadata| metadata.len()))
        .unwrap_or(0);

    Ok(MediaInfo {
        kind: if video.is_some() {
            MediaKind::Video
        } else if audio.is_some() {
            MediaKind::Audio
        } else {
            MediaKind::Unknown
        },
        duration_seconds,
        width: video.and_then(|stream| stream["width"].as_u64().map(|value| value as u32)),
        height: video.and_then(|stream| stream["height"].as_u64().map(|value| value as u32)),
        bitrate,
        file_size,
        video_codec: video.and_then(|stream| match stream["codec_name"].as_str()? {
            "h264" => Some(VideoCodec::H264),
            "hevc" | "h265" => Some(VideoCodec::H265),
            "vp9" => Some(VideoCodec::Vp9),
            "av1" => Some(VideoCodec::Av1),
            _ => None,
        }),
        audio_codec: audio.and_then(|stream| match stream["codec_name"].as_str()? {
            "aac" => Some(AudioCodec::Aac),
            "mp3" => Some(AudioCodec::Mp3),
            "opus" => Some(AudioCodec::Opus),
            "flac" => Some(AudioCodec::Flac),
            name if name.starts_with("pcm_") => Some(AudioCodec::Pcm),
            _ => None,
        }),
        image_codec: None,
    })
}

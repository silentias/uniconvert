use crate::{domain::ConvertError, infra::ffprobe};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::{fs, path::Path};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaMetadata {
    pub kind: MediaMetadataKind,
    pub format: String,
    pub file_size: u64,
    pub duration_seconds: Option<f64>,
    pub bitrate: Option<u64>,
    pub video: Option<VideoMetadata>,
    pub audio: Option<AudioMetadata>,
    pub tags: Vec<MediaMetadataField>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MediaMetadataKind {
    Audio,
    Video,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoMetadata {
    pub codec: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub frame_rate: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioMetadata {
    pub codec: String,
    pub sample_rate_hz: Option<u32>,
    pub channels: Option<u32>,
    pub channel_layout: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaMetadataField {
    pub key: String,
    pub value: String,
}

pub fn extract(path: &Path) -> Result<MediaMetadata, ConvertError> {
    let file_metadata = fs::metadata(path)
        .map_err(|error| ConvertError::InvalidInput(format!("{}: {error}", path.display())))?;
    if !file_metadata.is_file() {
        return Err(ConvertError::InvalidInput(
            "Ожидается локальный аудио- или видеофайл".into(),
        ));
    }

    let json: Value = serde_json::from_str(&ffprobe::probe_json(path)?)
        .map_err(|error| ConvertError::ProcessFailed(error.to_string()))?;
    parse(&json, file_metadata.len())
}

fn parse(json: &Value, fallback_file_size: u64) -> Result<MediaMetadata, ConvertError> {
    let streams = json["streams"].as_array().cloned().unwrap_or_default();
    let video_stream = streams.iter().find(|stream| {
        stream["codec_type"] == "video" && stream["disposition"]["attached_pic"] != 1
    });
    let audio_stream = streams
        .iter()
        .find(|stream| stream["codec_type"] == "audio");

    if video_stream.is_none() && audio_stream.is_none() {
        return Err(ConvertError::InvalidInput(
            "В файле не найдены аудио- или видеопотоки".into(),
        ));
    }

    let format = &json["format"];
    let duration_seconds = number(&format["duration"])
        .or_else(|| {
            streams
                .iter()
                .find_map(|stream| number(&stream["duration"]))
        })
        .filter(|value| value.is_finite() && *value >= 0.0);
    let bitrate = integer(&format["bit_rate"]);
    let file_size = integer(&format["size"]).unwrap_or(fallback_file_size);
    let format_name = text(&format["format_long_name"])
        .or_else(|| text(&format["format_name"]))
        .unwrap_or_else(|| "Неизвестный формат".into());

    let video = video_stream.map(|stream| VideoMetadata {
        codec: codec_name(stream),
        width: integer(&stream["width"]).and_then(|value| value.try_into().ok()),
        height: integer(&stream["height"]).and_then(|value| value.try_into().ok()),
        frame_rate: rational(&stream["avg_frame_rate"])
            .or_else(|| rational(&stream["r_frame_rate"])),
    });
    let audio = audio_stream.map(|stream| AudioMetadata {
        codec: codec_name(stream),
        sample_rate_hz: integer(&stream["sample_rate"]).and_then(|value| value.try_into().ok()),
        channels: integer(&stream["channels"]).and_then(|value| value.try_into().ok()),
        channel_layout: text(&stream["channel_layout"]),
    });

    let mut tags = Vec::new();
    collect_tags(&mut tags, format["tags"].as_object());
    for stream in &streams {
        collect_tags(&mut tags, stream["tags"].as_object());
    }

    Ok(MediaMetadata {
        kind: if video.is_some() {
            MediaMetadataKind::Video
        } else {
            MediaMetadataKind::Audio
        },
        format: format_name,
        file_size,
        duration_seconds,
        bitrate,
        video,
        audio,
        tags,
    })
}

fn codec_name(stream: &Value) -> String {
    text(&stream["codec_long_name"])
        .or_else(|| text(&stream["codec_name"]))
        .unwrap_or_else(|| "Неизвестен".into())
}

fn number(value: &Value) -> Option<f64> {
    value
        .as_f64()
        .or_else(|| value.as_str()?.parse::<f64>().ok())
}

fn integer(value: &Value) -> Option<u64> {
    value
        .as_u64()
        .or_else(|| value.as_str()?.parse::<u64>().ok())
}

fn rational(value: &Value) -> Option<f64> {
    let value = value.as_str()?;
    let (numerator, denominator) = value.split_once('/')?;
    let numerator = numerator.parse::<f64>().ok()?;
    let denominator = denominator.parse::<f64>().ok()?;
    (denominator != 0.0).then_some(numerator / denominator)
}

fn text(value: &Value) -> Option<String> {
    let value = value.as_str()?.trim();
    (!value.is_empty()).then(|| value.to_owned())
}

fn collect_tags(fields: &mut Vec<MediaMetadataField>, tags: Option<&Map<String, Value>>) {
    let Some(tags) = tags else { return };
    for (source_key, key) in [
        ("title", "title"),
        ("artist", "artist"),
        ("album_artist", "albumArtist"),
        ("album", "album"),
        ("genre", "genre"),
        ("date", "date"),
    ] {
        if fields.iter().any(|field| field.key == key) {
            continue;
        }
        let value = tags
            .iter()
            .find(|(candidate, _)| candidate.eq_ignore_ascii_case(source_key))
            .and_then(|(_, value)| text(value));
        if let Some(value) = value {
            fields.push(MediaMetadataField {
                key: key.into(),
                value,
            });
        }
    }
}

pub fn strip(path: &Path) -> Result<(), ConvertError> {
    // TODO: Сначала удалить необязательные текстовые теги.
    // TODO: Затем дать точный контроль над EXIF, GPS, главами и обложками.
    todo!("metadata stripping is not implemented")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn reads_compact_video_metadata_and_useful_tags() {
        let value = json!({
            "streams": [
                {"codec_type":"video","codec_name":"h264","codec_long_name":"H.264 / AVC","width":1920,"height":1080,"avg_frame_rate":"30000/1001","disposition":{"attached_pic":0}},
                {"codec_type":"audio","codec_name":"aac","sample_rate":"48000","channels":2,"channel_layout":"stereo","tags":{"artist":"Stream artist"}}
            ],
            "format": {"format_long_name":"QuickTime / MOV","duration":"12.5","size":"2048","bit_rate":"131072","tags":{"title":"Demo","artist":"Format artist","genre":"Rock"}}
        });

        let metadata = parse(&value, 1).unwrap();
        assert_eq!(metadata.kind, MediaMetadataKind::Video);
        assert_eq!(metadata.file_size, 2048);
        assert_eq!(metadata.video.as_ref().unwrap().width, Some(1920));
        assert!((metadata.video.unwrap().frame_rate.unwrap() - 29.97).abs() < 0.01);
        assert_eq!(metadata.audio.unwrap().sample_rate_hz, Some(48000));
        assert_eq!(metadata.tags[0].value, "Demo");
        assert_eq!(metadata.tags[1].value, "Format artist");
    }

    #[test]
    fn treats_album_art_as_audio_and_rejects_files_without_media_streams() {
        let audio = json!({
            "streams": [
                {"codec_type":"video","disposition":{"attached_pic":1}},
                {"codec_type":"audio","codec_name":"mp3","channels":2}
            ],
            "format": {"format_name":"mp3"}
        });
        assert_eq!(parse(&audio, 10).unwrap().kind, MediaMetadataKind::Audio);
        assert!(parse(&json!({"streams":[]}), 0).is_err());
    }
}

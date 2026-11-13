use std::path::PathBuf;

use serde::Serialize;
use uniconvert_core::{analysis::probe, domain::ConvertError};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputEstimate {
    estimated_bytes: u64,
    lower_bytes: u64,
    upper_bytes: u64,
}

#[tauri::command]
pub fn estimate_output(
    input: PathBuf,
    media_kind: String,
    extract_audio: Option<bool>,
    format: String,
    sample_rate_hz: Option<u32>,
    channels: Option<u32>,
    audio_bitrate_kbps: Option<u32>,
    video_codec: Option<String>,
    crf: Option<u8>,
    height: Option<u32>,
    copy_streams: bool,
    trim_start: Option<f64>,
    trim_end: Option<f64>,
    tempo: Option<f32>,
) -> Result<Option<OutputEstimate>, ConvertError> {
    if media_kind == "image" || (media_kind == "video" && !copy_streams) {
        return Ok(None);
    }
    if media_kind == "audio" {
        let info = probe::probe_audio_stream(
            &input,
            &std::sync::atomic::AtomicBool::new(false),
            extract_audio.unwrap_or(false),
        )?;
        let duration = uniconvert_core::analysis::audio_trim::trimmed_duration(
            trim_start,
            trim_end,
            info.duration_seconds,
        )?;
        let bytes = uniconvert_core::analysis::estimate::audio_size(
            &format,
            duration,
            tempo.unwrap_or(1.0),
            audio_bitrate_kbps,
            sample_rate_hz.or(info.sample_rate_hz),
            channels.or(info.channels),
        )?;
        return Ok(bytes.map(|bytes| OutputEstimate {
            estimated_bytes: bytes,
            lower_bytes: (bytes as f64 * 0.95) as u64,
            upper_bytes: (bytes as f64 * 1.10 + 8192.0) as u64,
        }));
    }
    // TODO: Позже добавить короткое пробное кодирование для точного режима оценки.
    let media = probe::probe(&input)?;
    let source = media.file_size.max(1) as f64;
    let estimated = if copy_streams {
        source
    } else if media_kind == "audio" {
        let duration = uniconvert_core::analysis::audio_trim::trimmed_duration(
            trim_start,
            trim_end,
            media.duration_seconds,
        )?;
        let tempo = tempo.unwrap_or(1.0);
        if !tempo.is_finite() || !(0.5..=2.0).contains(&tempo) {
            return Err(ConvertError::InvalidInput(
                "Темп должен быть от 0,5 до 2,0".into(),
            ));
        }
        let duration = duration.map(|value| value / tempo as f64);
        match (duration, audio_bitrate_kbps) {
            (Some(duration), Some(bitrate)) => duration * bitrate as f64 * 1_000.0 / 8.0 * 1.02,
            _ => {
                source
                    * 0.9
                    * match (duration, media.duration_seconds) {
                        (Some(selected), Some(full)) if full > 0.0 => selected / full,
                        _ => 1.0,
                    }
            }
        }
    } else if media_kind == "video" {
        let crf_factor = ((36.0 - crf.unwrap_or(23) as f64) / 20.0).clamp(0.25, 1.6);
        let codec_factor = match video_codec.as_deref() {
            Some("h265") => 0.75,
            Some("vp9") => 0.72,
            Some("av1") => 0.62,
            _ => 1.0,
        };
        let resolution_factor = match (height, media.height) {
            (Some(target), Some(source_height)) if source_height > 0 => (target as f64
                / source_height as f64)
                .powi(2)
                .clamp(0.08, 2.0),
            _ => 1.0,
        };
        source * crf_factor * codec_factor * resolution_factor
    } else {
        source
    };
    let uncertainty = if media_kind == "video" && !copy_streams {
        0.35
    } else {
        0.12
    };
    Ok(Some(OutputEstimate {
        estimated_bytes: estimated.max(1.0) as u64,
        lower_bytes: (estimated * (1.0 - uncertainty)).max(1.0) as u64,
        upper_bytes: (estimated * (1.0 + uncertainty)).max(1.0) as u64,
    }))
}

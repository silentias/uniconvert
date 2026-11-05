//! Shared audio workflow used by the desktop application and the CLI.
use crate::{
    analysis::{
        audio_trim::trimmed_duration,
        probe::{probe_audio, AudioInfo},
    },
    domain::{CompressionMode, Container, ConvertError, ConvertOptions, ConvertRequest},
    infra::paths::available_output_path,
};
use serde::{Deserialize, Serialize};
use std::{
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AudioOptions {
    pub format: String,
    pub bitrate_kbps: Option<u32>,
    pub sample_rate_hz: Option<u32>,
    pub channels: Option<u8>,
    pub volume_db: Option<f32>,
    pub tempo: Option<f32>,
    pub pitch_semitones: Option<f32>,
    pub normalize_lufs: bool,
    pub target_lufs: Option<f32>,
    pub strip_metadata: bool,
    pub trim_start: Option<f64>,
    pub trim_end: Option<f64>,
}

impl AudioOptions {
    pub fn convert_options(&self) -> Result<ConvertOptions, ConvertError> {
        let container = match self.format.to_ascii_lowercase().as_str() {
            "mp3" => Container::Mp3,
            "wav" => Container::Wav,
            "flac" => Container::Flac,
            "ogg" => Container::Ogg,
            "aac" => Container::Aac,
            "opus" => Container::Opus,
            "m4a" => Container::M4a,
            "wma" => Container::Wma,
            "aiff" | "aif" => Container::Aiff,
            _ => return Err(ConvertError::UnsupportedFormat(self.format.clone())),
        };
        let invalid = |message: &str| ConvertError::InvalidInput(message.into());
        if self
            .bitrate_kbps
            .is_some_and(|value| !(8..=1536).contains(&value))
        {
            return Err(invalid("Битрейт должен быть от 8 до 1536 кбит/с"));
        }
        if self.bitrate_kbps.is_some()
            && matches!(
                container,
                Container::Wav | Container::Flac | Container::Aiff
            )
        {
            return Err(invalid("Для WAV, FLAC и AIFF битрейт не задаётся"));
        }
        if self
            .sample_rate_hz
            .is_some_and(|value| !(8000..=384000).contains(&value))
        {
            return Err(invalid(
                "Частота дискретизации должна быть от 8000 до 384000 Гц",
            ));
        }
        if self.channels.is_some_and(|value| ![1, 2].contains(&value)) {
            return Err(invalid("Количество каналов должно быть 1 или 2"));
        }
        if self
            .volume_db
            .is_some_and(|value| !value.is_finite() || !(-60.0..=30.0).contains(&value))
        {
            return Err(invalid("Громкость должна быть от -60 до 30 дБ"));
        }
        if self
            .tempo
            .is_some_and(|value| !value.is_finite() || !(0.5..=2.0).contains(&value))
        {
            return Err(invalid("Темп должен быть от 0,5 до 2,0"));
        }
        if self.pitch_semitones.is_some_and(|value| {
            !value.is_finite() || !(-12.0..=12.0).contains(&value)
        }) {
            return Err(invalid("Высота тона должна быть от -12 до 12 полутонов"));
        }
        if self.target_lufs.is_some() && !self.normalize_lufs {
            return Err(invalid(
                "Целевой LUFS задаётся только вместе с нормализацией",
            ));
        }
        if self
            .target_lufs
            .is_some_and(|value| !value.is_finite() || !(-70.0..=-5.0).contains(&value))
        {
            return Err(invalid("Целевой уровень должен быть от -70 до -5 LUFS"));
        }
        trimmed_duration(self.trim_start, self.trim_end, None)?;
        Ok(ConvertOptions {
            container,
            video_codec: None,
            audio_codec: None,
            compression: CompressionMode::Custom,
            quality: None,
            strip_metadata: self.strip_metadata,
            audio_bitrate_kbps: self.bitrate_kbps,
            audio_sample_rate_hz: self.sample_rate_hz,
            audio_channels: self.channels,
            audio_volume_db: self.volume_db,
            audio_tempo: self.tempo,
            audio_pitch_semitones: self.pitch_semitones,
            audio_normalize_lufs: self.normalize_lufs,
            audio_target_lufs: self.target_lufs,
            audio_trim_start: self.trim_start,
            audio_trim_end: self.trim_end,
            video_crf: None,
            video_height: None,
            video_fps: None,
            copy_streams: false,
        })
    }
}

pub struct PreparedAudio {
    pub request: ConvertRequest,
    pub info: AudioInfo,
}

pub fn prepare_conversion(
    input: &Path,
    output_directory: Option<&Path>,
    options: &AudioOptions,
    cancelled: &AtomicBool,
) -> Result<PreparedAudio, ConvertError> {
    let options = options.convert_options()?;
    if cancelled.load(Ordering::Relaxed) {
        return Err(ConvertError::Cancelled);
    }
    let input = std::fs::canonicalize(input)
        .map_err(|error| ConvertError::InvalidInput(format!("{}: {error}", input.display())))?;
    let info = probe_audio(&input, cancelled)?;
    trimmed_duration(
        options.audio_trim_start,
        options.audio_trim_end,
        info.duration_seconds,
    )?;
    if cancelled.load(Ordering::Relaxed) {
        return Err(ConvertError::Cancelled);
    }
    let directory = output_directory
        .map(PathBuf::from)
        .unwrap_or_else(|| input.parent().unwrap().to_owned());
    std::fs::create_dir_all(&directory).map_err(|error| ConvertError::Io(error.to_string()))?;
    let directory =
        std::fs::canonicalize(directory).map_err(|error| ConvertError::Io(error.to_string()))?;
    let output = available_output_path(&input, &directory, options.container)?;
    Ok(PreparedAudio {
        request: ConvertRequest {
            input,
            output,
            options,
        },
        info,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn validates_shared_audio_settings() {
        let mut settings = AudioOptions {
            format: "MP3".into(),
            ..Default::default()
        };
        assert_eq!(
            settings.convert_options().unwrap().container,
            Container::Mp3
        );
        settings.volume_db = Some(f32::NAN);
        assert!(settings.convert_options().is_err());
        settings.volume_db = None;
        settings.tempo = Some(0.49);
        assert!(settings.convert_options().is_err());
        settings.tempo = Some(1.25);
        settings.pitch_semitones = Some(13.0);
        assert!(settings.convert_options().is_err());
        settings.pitch_semitones = Some(-4.0);
        settings.target_lufs = Some(-16.0);
        assert!(settings.convert_options().is_err());
        settings.normalize_lufs = true;
        assert!(settings.convert_options().is_ok());
        settings.channels = Some(3);
        assert!(settings.convert_options().is_err());
        settings.channels = None;
        settings.format = "wav".into();
        settings.bitrate_kbps = Some(192);
        assert!(settings.convert_options().is_err());
        settings.bitrate_kbps = None;
        settings.trim_start = Some(5.0);
        settings.trim_end = Some(4.0);
        assert!(settings.convert_options().is_err());
    }
}

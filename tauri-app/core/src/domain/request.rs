use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::{AudioCodec, CompressionMode, Container, VideoCodec};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvertOptions {
    pub container: Container,
    pub video_codec: Option<VideoCodec>,
    pub audio_codec: Option<AudioCodec>,
    pub compression: CompressionMode,
    pub quality: Option<u8>,
    pub strip_metadata: bool,
    pub audio_bitrate_kbps: Option<u32>,
    pub audio_sample_rate_hz: Option<u32>,
    pub audio_channels: Option<u8>,
    pub audio_volume_db: Option<f32>,
    pub audio_tempo: Option<f32>,
    pub audio_pitch_semitones: Option<f32>,
    pub audio_normalize_lufs: bool,
    pub audio_target_lufs: Option<f32>,
    pub audio_trim_start: Option<f64>,
    pub audio_trim_end: Option<f64>,
    pub video_crf: Option<u8>,
    pub video_height: Option<u32>,
    pub video_fps: Option<u32>,
    pub copy_streams: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvertRequest {
    pub input: PathBuf,
    pub output: PathBuf,
    pub options: ConvertOptions,
}

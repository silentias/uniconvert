use std::path::PathBuf;

use tauri::{AppHandle, Emitter, State};

use uniconvert_core::{
    converters::VideoConverter,
    domain::{
        AudioCodec, CompressionMode, Container, ConvertError, ConvertOptions, ConvertOutput,
        ConvertRequest, VideoCodec,
    },
    infra::paths::available_output_path,
};

use crate::state::{ActiveJobs, ConversionProgressEvent};

#[tauri::command]
pub async fn convert_video(
    app: AppHandle,
    state: State<'_, ActiveJobs>,
    job_id: String,
    input: PathBuf,
    output_directory: Option<PathBuf>,
    format: String,
    video_codec: String,
    crf: u8,
    height: Option<u32>,
    fps: Option<u32>,
    audio_codec: String,
    audio_bitrate_kbps: Option<u32>,
    strip_metadata: bool,
    copy_streams: bool,
) -> Result<ConvertOutput, ConvertError> {
    // TODO: Затем добавить пресеты скорости, HDR и аппаратное ускорение.
    let cancelled = state.register(job_id.clone());
    let event_job_id = job_id.clone();
    let result = tauri::async_runtime::spawn_blocking(move || {
        let container = match format.to_ascii_lowercase().as_str() {
            "mp4" => Container::Mp4,
            "mkv" => Container::Mkv,
            "webm" => Container::Webm,
            "mov" => Container::Mov,
            "avi" => Container::Avi,
            _ => return Err(ConvertError::UnsupportedFormat(format)),
        };
        let video_codec = match video_codec.to_ascii_lowercase().as_str() {
            "h264" => VideoCodec::H264,
            "h265" => VideoCodec::H265,
            "vp9" => VideoCodec::Vp9,
            "av1" => VideoCodec::Av1,
            _ => return Err(ConvertError::InvalidInput("Неизвестный видеокодек".into())),
        };
        let audio_codec = match audio_codec.to_ascii_lowercase().as_str() {
            "aac" => AudioCodec::Aac,
            "mp3" => AudioCodec::Mp3,
            "opus" => AudioCodec::Opus,
            "flac" => AudioCodec::Flac,
            _ => return Err(ConvertError::InvalidInput("Неизвестный аудиокодек".into())),
        };
        if crf > 51 {
            return Err(ConvertError::InvalidInput(
                "CRF должен быть в диапазоне от 0 до 51".into(),
            ));
        }
        if let Some(value) = height {
            if ![480, 720, 1080, 1440, 2160].contains(&value) {
                return Err(ConvertError::InvalidInput("Недопустимое разрешение".into()));
            }
        }
        if let Some(value) = fps {
            if ![24, 30, 50, 60].contains(&value) {
                return Err(ConvertError::InvalidInput(
                    "Недопустимая частота кадров".into(),
                ));
            }
        }
        if let Some(value) = audio_bitrate_kbps {
            if !(8..=1536).contains(&value) {
                return Err(ConvertError::InvalidInput(
                    "Битрейт аудио должен быть от 8 до 1536 кбит/с".into(),
                ));
            }
        }
        let directory = output_directory
            .or_else(|| input.parent().map(PathBuf::from))
            .ok_or_else(|| {
                ConvertError::InvalidInput("Не удалось определить папку результата".into())
            })?;
        std::fs::create_dir_all(&directory).map_err(|error| ConvertError::Io(error.to_string()))?;
        let output = available_output_path(&input, &directory, container)?;

        let request = ConvertRequest {
            input,
            output,
            options: ConvertOptions {
                container,
                video_codec: Some(video_codec),
                audio_codec: Some(audio_codec),
                compression: CompressionMode::Custom,
                quality: None,
                strip_metadata,
                audio_bitrate_kbps,
                audio_sample_rate_hz: None,
                audio_channels: None,
                audio_volume_db: None,
                audio_tempo: None,
                audio_pitch_semitones: None,
                audio_normalize_lufs: false,
                audio_target_lufs: None,
                audio_trim_start: None,
                audio_trim_end: None,
                video_crf: Some(crf),
                video_height: height,
                video_fps: fps,
                copy_streams,
            },
        };
        let duration = uniconvert_core::analysis::probe::probe(&request.input)
            .ok()
            .and_then(|media| media.duration_seconds);
        VideoConverter.convert_with_progress(&request, duration, &cancelled, |percent| {
            let _ = app.emit(
                "conversion-progress",
                ConversionProgressEvent {
                    job_id: event_job_id.clone(),
                    percent,
                },
            );
        })
    })
    .await
    .map_err(|error| ConvertError::ProcessFailed(error.to_string()))?;
    state.remove(&job_id);
    result
}

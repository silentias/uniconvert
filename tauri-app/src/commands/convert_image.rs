use std::path::PathBuf;

use uniconvert_core::{
    converters::{image::ImageConverter, Converter},
    domain::{
        CompressionMode, Container, ConvertError, ConvertOptions, ConvertOutput, ConvertRequest,
    },
    infra::paths::available_output_path,
};

#[tauri::command]
pub async fn convert_image(
    input: PathBuf,
    output_directory: Option<PathBuf>,
    format: String,
    quality: u8,
    strip_metadata: bool,
) -> Result<ConvertOutput, ConvertError> {
    // TODO: Сейчас конвертировать только расширение; затем принять качество, размер и метаданные.
    tauri::async_runtime::spawn_blocking(move || {
        let container = Container::from_image_extension(&format)
            .ok_or_else(|| ConvertError::UnsupportedFormat(format))?;
        if quality > 100 {
            return Err(ConvertError::InvalidInput(
                "Качество должно быть в диапазоне от 0 до 100".into(),
            ));
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
                video_codec: None,
                audio_codec: None,
                compression: CompressionMode::Custom,
                quality: Some(quality),
                strip_metadata,
                audio_bitrate_kbps: None,
                audio_sample_rate_hz: None,
                audio_channels: None,
                audio_volume_db: None,
                audio_tempo: None,
                audio_pitch_semitones: None,
                audio_normalize_lufs: false,
                audio_target_lufs: None,
                audio_trim_start: None,
                audio_trim_end: None,
                video_crf: None,
                video_height: None,
                video_fps: None,
                copy_streams: false,
            },
        };

        ImageConverter.convert(&request)
    })
    .await
    .map_err(|error| ConvertError::ProcessFailed(error.to_string()))?
}

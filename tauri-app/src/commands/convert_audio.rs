use std::path::PathBuf;

use tauri::{AppHandle, Emitter, State};

use uniconvert_core::{
    audio::{prepare_conversion, AudioOptions},
    converters::AudioConverter,
    domain::{ConvertError, ConvertOutput},
};

use crate::state::{ActiveJobs, ConversionProgressEvent};

#[tauri::command]
pub async fn convert_audio(
    app: AppHandle,
    state: State<'_, ActiveJobs>,
    job_id: String,
    input: PathBuf,
    output_directory: Option<PathBuf>,
    format: String,
    bitrate_kbps: Option<u32>,
    sample_rate_hz: Option<u32>,
    channels: Option<u8>,
    volume_db: Option<f32>,
    tempo: Option<f32>,
    pitch_semitones: Option<f32>,
    normalize_lufs: bool,
    target_lufs: Option<f32>,
    strip_metadata: bool,
    trim_start: Option<f64>,
    trim_end: Option<f64>,
) -> Result<ConvertOutput, ConvertError> {
    let cancelled = state.register(job_id.clone());
    let event_job_id = job_id.clone();
    let result = tauri::async_runtime::spawn_blocking(move || {
        let options = AudioOptions {
            format,
            bitrate_kbps,
            sample_rate_hz,
            channels,
            volume_db,
            tempo,
            pitch_semitones,
            normalize_lufs,
            target_lufs,
            strip_metadata,
            trim_start,
            trim_end,
        };
        let prepared =
            prepare_conversion(&input, output_directory.as_deref(), &options, &cancelled)?;
        AudioConverter.convert_with_progress(
            &prepared.request,
            prepared.info.duration_seconds,
            &cancelled,
            |percent| {
                let _ = app.emit(
                    "conversion-progress",
                    ConversionProgressEvent {
                        job_id: event_job_id.clone(),
                        percent,
                    },
                );
            },
        )
    })
    .await;
    state.remove(&job_id);
    result.map_err(|error| ConvertError::ProcessFailed(error.to_string()))?
}

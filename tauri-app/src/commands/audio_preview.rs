use crate::state::{ActiveJobs, ConversionProgressEvent};
use serde::Serialize;
use std::{collections::HashMap, path::PathBuf, sync::Mutex};
use tauri::{AppHandle, Emitter, Manager, State};
use uniconvert_core::{analysis::waveform, domain::ConvertError};

#[derive(Default)]
pub struct AudioPreviews(Mutex<HashMap<String, tempfile::TempDir>>);

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewResponse {
    path: PathBuf,
    duration_seconds: f64,
    peaks: Vec<f32>,
}

#[tauri::command]
pub async fn prepare_audio_preview(
    app: AppHandle,
    state: State<'_, ActiveJobs>,
    previews: State<'_, AudioPreviews>,
    job_id: String,
    input: PathBuf,
) -> Result<PreviewResponse, ConvertError> {
    let cancelled = state.register(job_id.clone());
    let event_id = job_id.clone();
    let event_app = app.clone();
    let result = tauri::async_runtime::spawn_blocking(move || {
        let directory = tempfile::Builder::new()
            .prefix("uniconvert-audio-")
            .tempdir()
            .map_err(|error| ConvertError::Io(error.to_string()))?;
        let path = directory.path().join("preview.wav");
        let preview = waveform::prepare_audio_preview(&input, &path, &cancelled, |percent| {
            let _ = event_app.emit(
                "audio-preview-progress",
                ConversionProgressEvent {
                    job_id: event_id.clone(),
                    percent,
                },
            );
        })?;
        Ok::<_, ConvertError>((
            directory,
            PreviewResponse {
                path,
                duration_seconds: preview.duration_seconds,
                peaks: preview.peaks,
            },
        ))
    })
    .await;
    state.remove(&job_id);
    let (directory, response) =
        result.map_err(|error| ConvertError::ProcessFailed(error.to_string()))??;
    app.asset_protocol_scope()
        .allow_file(&response.path)
        .map_err(|error| ConvertError::Io(error.to_string()))?;
    previews
        .0
        .lock()
        .map_err(|error| ConvertError::Io(error.to_string()))?
        .insert(job_id, directory);
    Ok(response)
}

#[tauri::command]
pub fn release_audio_preview(
    app: AppHandle,
    previews: State<'_, AudioPreviews>,
    job_id: String,
) -> Result<(), ConvertError> {
    if let Some(directory) = previews
        .0
        .lock()
        .map_err(|error| ConvertError::Io(error.to_string()))?
        .remove(&job_id)
    {
        let _ = app
            .asset_protocol_scope()
            .forbid_file(directory.path().join("preview.wav"));
        directory
            .close()
            .map_err(|error| ConvertError::Io(error.to_string()))?;
    }
    Ok(())
}

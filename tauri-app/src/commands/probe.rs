use std::path::PathBuf;
use uniconvert_core::{
    analysis::metadata::{self, MediaMetadata},
    domain::ConvertError,
};

#[tauri::command]
pub async fn probe_file(path: PathBuf) -> Result<MediaMetadata, ConvertError> {
    tauri::async_runtime::spawn_blocking(move || metadata::extract(&path))
        .await
        .map_err(|error| ConvertError::ProcessFailed(error.to_string()))?
}

use std::path::PathBuf;

use uniconvert_core::{
    analysis::image_metadata::{self, ImageMetadata},
    domain::ConvertError,
};

#[tauri::command]
pub async fn extract_image_metadata(path: PathBuf) -> Result<ImageMetadata, ConvertError> {
    tauri::async_runtime::spawn_blocking(move || image_metadata::extract_metadata(&path))
        .await
        .map_err(|error| ConvertError::ProcessFailed(error.to_string()))?
}

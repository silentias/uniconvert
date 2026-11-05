use std::path::PathBuf;
use uniconvert_core::domain::ConvertError;

#[tauri::command]
pub fn generate_preview(input: PathBuf, timestamp: f64) -> Result<PathBuf, ConvertError> {
    // TODO: Сначала создать один временный thumbnail.
    // TODO: Затем поддержать сниппеты, waveform и автоматический выбор сцены.
    todo!("preview command is not implemented")
}

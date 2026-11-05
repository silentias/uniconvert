use uniconvert_core::domain::{Container, ConvertError, MediaKind};

#[tauri::command]
pub fn list_supported_formats(kind: MediaKind) -> Result<Vec<Container>, ConvertError> {
    // TODO: Сначала вернуть контейнеры для типа медиа.
    // TODO: Затем отфильтровать их по кодекам и возможностям FFmpeg.
    todo!("formats command is not implemented")
}

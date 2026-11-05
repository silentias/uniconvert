use uniconvert_core::domain::{ConvertError, MediaKind, Preset};

#[tauri::command]
pub fn list_presets(kind: MediaKind) -> Result<Vec<Preset>, ConvertError> {
    // TODO: Сначала вернуть встроенные пресеты нужного типа.
    // TODO: Затем объединить пользовательские и системно-доступные пресеты.
    todo!("preset command is not implemented")
}

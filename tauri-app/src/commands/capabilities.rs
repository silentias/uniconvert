use uniconvert_core::{domain::ConvertError, infra::hwaccel::HardwareEncoder};

#[tauri::command]
pub fn detect_capabilities() -> Result<Vec<HardwareEncoder>, ConvertError> {
    // TODO: Сначала вернуть доступные GPU-энкодеры.
    // TODO: Затем добавить кодеки, версии бинарников и системные лимиты.
    todo!("capabilities command is not implemented")
}

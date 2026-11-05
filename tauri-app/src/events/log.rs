use tauri::AppHandle;
use uniconvert_core::domain::ConvertError;

pub fn emit(app: &AppHandle, level: &str, message: &str) -> Result<(), ConvertError> {
    // TODO: Сначала отправить диагностическое событие convert://log.
    // TODO: Затем добавить структурированные поля, фильтрацию и redaction.
    todo!("log event transport is not implemented")
}

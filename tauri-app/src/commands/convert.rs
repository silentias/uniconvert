use uniconvert_core::{
    domain::{ConvertError, ConvertRequest},
    jobs::JobId,
};

#[tauri::command]
pub fn convert(request: ConvertRequest) -> Result<JobId, ConvertError> {
    // TODO: Сначала создать Job и вернуть его ID.
    // TODO: Затем поставить Job в очередь и разбудить scheduler.
    // TODO: Позже поддержать batch и зависимости задач.
    todo!("convert command is not implemented")
}

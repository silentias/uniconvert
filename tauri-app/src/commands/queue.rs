use uniconvert_core::{
    domain::ConvertError,
    jobs::{Job, JobId},
};

#[tauri::command]
pub fn get_queue() -> Result<Vec<Job>, ConvertError> {
    // TODO: Сначала вернуть снимок очереди.
    // TODO: Затем добавить пагинацию и подписку на изменения.
    todo!("get queue command is not implemented")
}

#[tauri::command]
pub fn clear_queue() -> Result<(), ConvertError> {
    // TODO: Сначала очистить ожидающие задачи.
    // TODO: Затем принимать политику обработки активных задач.
    todo!("clear queue command is not implemented")
}

#[tauri::command]
pub fn reorder_queue(order: Vec<JobId>) -> Result<(), ConvertError> {
    // TODO: Сначала применить новый порядок ожидающих Job.
    // TODO: Затем валидировать гонки и приоритеты scheduler.
    todo!("reorder queue command is not implemented")
}

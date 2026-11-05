use tauri::AppHandle;
use uniconvert_core::{
    domain::ConvertError,
    jobs::{JobId, JobStatus},
};

pub fn emit(app: &AppHandle, job_id: JobId, status: JobStatus) -> Result<(), ConvertError> {
    // TODO: Сначала отправить convert://job-changed.
    // TODO: Затем приложить результат, ошибку и временные метки.
    todo!("job status event transport is not implemented")
}

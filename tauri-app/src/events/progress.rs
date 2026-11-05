use tauri::AppHandle;
use uniconvert_core::{
    domain::{ConvertError, Progress},
    jobs::JobId,
};

pub fn emit(app: &AppHandle, job_id: JobId, progress: Progress) -> Result<(), ConvertError> {
    // TODO: Сначала отправить сериализуемое событие convert://progress.
    // TODO: Затем ограничить частоту событий и объединять прогресс pipeline.
    todo!("progress event transport is not implemented")
}

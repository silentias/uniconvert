use tauri::AppHandle;
use uniconvert_core::{domain::ConvertError, jobs::Job};

pub fn emit(app: &AppHandle, jobs: &[Job]) -> Result<(), ConvertError> {
    // TODO: Сначала отправить полный снимок convert://queue-changed.
    // TODO: Затем перейти на версии и компактные изменения очереди.
    todo!("queue event transport is not implemented")
}

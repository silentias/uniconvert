use tauri::State;
use uniconvert_core::domain::ConvertError;

use crate::state::ActiveJobs;

#[tauri::command]
pub fn cancel_convert(job_id: String, state: State<'_, ActiveJobs>) -> Result<(), ConvertError> {
    // TODO: Позже отправлять отдельное подтверждение завершения процесса.
    if state.cancel(&job_id) {
        Ok(())
    } else {
        Err(ConvertError::InvalidInput(
            "Активная задача не найдена".into(),
        ))
    }
}

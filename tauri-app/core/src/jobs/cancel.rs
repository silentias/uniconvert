use super::JobId;
use crate::domain::ConvertError;

pub struct CancellationRegistry;

impl CancellationRegistry {
    pub fn cancel(&self, id: JobId) -> Result<(), ConvertError> {
        // TODO: Сначала отметить задачу отменённой.
        // TODO: Затем остановить активный процесс и очистить временные файлы.
        todo!("job cancellation is not implemented")
    }
}

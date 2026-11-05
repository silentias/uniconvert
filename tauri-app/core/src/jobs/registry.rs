use super::{Job, JobId, JobStatus};
use crate::domain::ConvertError;

pub struct JobRegistry;

impl JobRegistry {
    pub fn get(&self, id: JobId) -> Result<Option<Job>, ConvertError> {
        // TODO: Сначала найти задачу в памяти.
        // TODO: Затем добавить потокобезопасные обновления и снимки.
        todo!("job lookup is not implemented")
    }
    pub fn set_status(&self, id: JobId, status: JobStatus) -> Result<(), ConvertError> {
        // TODO: Сначала изменить статус существующей задачи.
        // TODO: Затем проверять допустимые переходы и сохранять историю.
        todo!("job status update is not implemented")
    }
}

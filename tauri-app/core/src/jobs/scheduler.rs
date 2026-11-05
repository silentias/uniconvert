use super::JobId;
use crate::domain::ConvertError;

pub fn next() -> Result<Option<JobId>, ConvertError> {
    // TODO: Сначала выбирать следующую FIFO-задачу.
    // TODO: Затем учитывать CPU/GPU слоты и приоритеты.
    // TODO: Позже предотвращать starvation и планировать batch.
    todo!("job scheduling is not implemented")
}

use super::JobId;
use crate::domain::ConvertError;

pub fn retry(id: JobId) -> Result<JobId, ConvertError> {
    // TODO: Сначала повторно поставить упавшую задачу в очередь.
    // TODO: Затем добавить backoff, лимиты и классификацию ошибок.
    todo!("job retry is not implemented")
}

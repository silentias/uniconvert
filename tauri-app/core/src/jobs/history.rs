use super::Job;
use crate::domain::ConvertError;

pub fn list() -> Result<Vec<Job>, ConvertError> {
    // TODO: Сначала вернуть завершённые задачи текущей сессии.
    // TODO: Затем хранить историю на диске, фильтровать и очищать её.
    todo!("job history is not implemented")
}

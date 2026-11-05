use super::{Job, JobId};
use crate::domain::ConvertError;

pub struct JobQueue;

impl JobQueue {
    pub fn enqueue(&self, job: Job) -> Result<(), ConvertError> {
        // TODO: Сначала добавить FIFO-очередь.
        // TODO: Затем ограничить параллелизм и добавить персистентность.
        todo!("queue enqueue is not implemented")
    }
    pub fn reorder(&self, order: &[JobId]) -> Result<(), ConvertError> {
        // TODO: Сначала переставить ожидающие задачи.
        // TODO: Затем учесть приоритеты и зависимые задачи.
        todo!("queue reorder is not implemented")
    }
    pub fn clear(&self) -> Result<(), ConvertError> {
        // TODO: Сначала удалить ожидающие задачи.
        // TODO: Затем поддержать политики для активных и завершённых задач.
        todo!("queue clearing is not implemented")
    }
}

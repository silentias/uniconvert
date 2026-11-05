pub mod cancel;
pub mod history;
pub mod queue;
pub mod registry;
pub mod retry;
pub mod scheduler;

use crate::domain::{ConvertError, ConvertRequest, Progress};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct JobId(pub u64);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobStatus {
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: JobId,
    pub request: ConvertRequest,
    pub status: JobStatus,
    pub progress: Option<Progress>,
    pub error: Option<ConvertError>,
}

pub fn create(request: ConvertRequest) -> Job {
    // TODO: Сначала создать Job с уникальным ID и статусом Queued.
    // TODO: Затем добавить приоритет, timestamps и группы batch.
    todo!("job creation is not implemented")
}

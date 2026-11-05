use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProgressStage {
    Queued,
    Analyzing,
    Converting,
    Finalizing,
    Completed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Progress {
    pub percent: f32,
    pub stage: ProgressStage,
    pub eta_seconds: Option<u64>,
    pub speed: Option<f32>,
}

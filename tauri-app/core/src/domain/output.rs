use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvertOutput {
    pub path: PathBuf,
    pub input_size: u64,
    pub output_size: u64,
    pub elapsed_millis: u64,
}

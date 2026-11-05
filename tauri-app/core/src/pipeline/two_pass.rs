use super::Pipeline;
use crate::domain::{ConvertError, ConvertOutput, ConvertRequest};
pub struct TwoPassPipeline;
impl Pipeline for TwoPassPipeline {
    fn execute(&self, request: &ConvertRequest) -> Result<ConvertOutput, ConvertError> {
        // TODO: Сначала последовательно выполнить анализ и кодирование.
        // TODO: Затем свести прогресс шагов и очищать passlog.
        todo!("two-pass pipeline is not implemented")
    }
}

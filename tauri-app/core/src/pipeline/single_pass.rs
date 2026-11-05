use super::Pipeline;
use crate::domain::{ConvertError, ConvertOutput, ConvertRequest};
pub struct SinglePassPipeline;
impl Pipeline for SinglePassPipeline {
    fn execute(&self, request: &ConvertRequest) -> Result<ConvertOutput, ConvertError> {
        // TODO: Сначала выполнить один запуск FFmpeg.
        // TODO: Затем добавить прогресс, отмену и атомарный результат.
        todo!("single-pass pipeline is not implemented")
    }
}

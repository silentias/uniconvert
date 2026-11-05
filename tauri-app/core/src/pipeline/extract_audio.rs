use super::Pipeline;
use crate::domain::{ConvertError, ConvertOutput, ConvertRequest};
pub struct ExtractAudioPipeline;
impl Pipeline for ExtractAudioPipeline {
    fn execute(&self, request: &ConvertRequest) -> Result<ConvertOutput, ConvertError> {
        // TODO: Сначала извлечь первую аудиодорожку.
        // TODO: Затем поддержать выбор дорожек и последующее сжатие.
        todo!("audio extraction pipeline is not implemented")
    }
}

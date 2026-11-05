use super::Pipeline;
use crate::domain::{ConvertError, ConvertOutput, ConvertRequest};
pub struct NormalizePipeline;
impl Pipeline for NormalizePipeline {
    fn execute(&self, request: &ConvertRequest) -> Result<ConvertOutput, ConvertError> {
        // TODO: Сначала измерить и применить loudness normalization.
        // TODO: Затем поддержать разные стандарты и true peak.
        todo!("normalization pipeline is not implemented")
    }
}

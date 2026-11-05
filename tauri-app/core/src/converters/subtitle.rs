use super::Converter;
use crate::domain::{ConvertError, ConvertOutput, ConvertRequest, MediaKind};

pub struct SubtitleConverter;

impl Converter for SubtitleConverter {
    fn supports(&self, kind: MediaKind, request: &ConvertRequest) -> bool {
        // TODO: Сначала распознавать SRT, VTT и ASS.
        // TODO: Затем проверять поддержку в целевом контейнере.
        todo!("subtitle support detection is not implemented")
    }

    fn convert(&self, request: &ConvertRequest) -> Result<ConvertOutput, ConvertError> {
        // TODO: Сначала конвертировать текстовые субтитры.
        // TODO: Затем добавить встраивание и прожиг в видео.
        todo!("subtitle conversion is not implemented")
    }
}

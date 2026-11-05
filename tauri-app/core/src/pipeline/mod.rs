mod batch;
mod concat;
mod extract_audio;
mod normalize;
mod single_pass;
mod two_pass;

use crate::domain::{ConvertError, ConvertOutput, ConvertRequest};

pub trait Pipeline: Send + Sync {
    fn execute(&self, request: &ConvertRequest) -> Result<ConvertOutput, ConvertError>;
}

pub fn select(request: &ConvertRequest) -> Box<dyn Pipeline> {
    // TODO: Сначала выбирать single-pass по умолчанию.
    // TODO: Затем учитывать target size и составные операции.
    // TODO: Позже строить динамический граф шагов.
    todo!("pipeline selection is not implemented")
}

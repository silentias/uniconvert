use crate::domain::ConvertError;

pub fn available_codecs() -> Result<Vec<String>, ConvertError> {
    // TODO: Сначала разобрать список кодеков FFmpeg.
    // TODO: Затем определить encode/decode и аппаратные варианты.
    todo!("codec probing is not implemented")
}

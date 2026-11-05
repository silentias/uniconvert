use crate::domain::{ConvertError, Progress};

pub fn parse(line: &str, duration_seconds: Option<f64>) -> Result<Option<Progress>, ConvertError> {
    // TODO: Сначала разобрать out_time и percentage.
    // TODO: Затем вычислять ETA, speed и этапы pipeline.
    todo!("FFmpeg progress parsing is not implemented")
}

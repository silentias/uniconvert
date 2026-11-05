use crate::domain::ConvertError;
use std::path::Path;

pub fn detect_scenes(path: &Path) -> Result<Vec<f64>, ConvertError> {
    // TODO: Сначала получить временные метки резких смен кадров.
    // TODO: Затем настраивать чувствительность и объединять близкие сцены.
    todo!("scene detection is not implemented")
}

use crate::domain::ConvertError;
use std::path::{Path, PathBuf};

pub fn generate_thumbnail(
    input: &Path,
    timestamp: f64,
    output: &Path,
) -> Result<PathBuf, ConvertError> {
    // TODO: Сначала извлечь один кадр в заданный момент.
    // TODO: Затем добавить авто-выбор кадра, размер и формат.
    todo!("thumbnail generation is not implemented")
}

use crate::domain::{ConvertError, ConvertOptions, MediaInfo};

pub fn estimate_size_precisely(
    media: &MediaInfo,
    options: &ConvertOptions,
) -> Result<u64, ConvertError> {
    // TODO: Сначала выполнить короткий пробный прогон.
    // TODO: Затем выбирать репрезентативные сцены и доверительный интервал.
    todo!("precise size estimation is not implemented")
}

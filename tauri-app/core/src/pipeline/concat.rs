use crate::domain::{ConvertError, ConvertOptions, ConvertOutput};
use std::path::PathBuf;

pub fn concatenate(
    inputs: &[PathBuf],
    output: PathBuf,
    options: &ConvertOptions,
) -> Result<ConvertOutput, ConvertError> {
    // TODO: Сначала склеивать совместимые файлы без перекодирования.
    // TODO: Затем нормализовать различающиеся потоки и таймбазы.
    todo!("concat pipeline is not implemented")
}

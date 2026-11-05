use crate::domain::Container;
use crate::domain::ConvertError;
use std::path::{Path, PathBuf};

pub fn normalize(path: &Path) -> Result<PathBuf, ConvertError> {
    // TODO: Сначала нормализовать абсолютный путь без потери Unicode.
    // TODO: Затем учесть UNC, symlink и особенности каждой ОС.
    todo!("path normalization is not implemented")
}

pub fn available_output_path(
    input: &Path,
    directory: &Path,
    container: Container,
) -> Result<PathBuf, ConvertError> {
    // TODO: Сейчас предотвращать перезапись суффиксом; позже добавить настраиваемую политику конфликтов.
    let stem = input
        .file_stem()
        .and_then(|value| value.to_str())
        .ok_or_else(|| ConvertError::InvalidInput("Не удалось определить имя файла".into()))?;
    let extension = container.extension();
    let direct = directory.join(format!("{stem}.{extension}"));

    if direct != input && !direct.exists() {
        return Ok(direct);
    }

    for index in 1..=10_000 {
        let suffix = if index == 1 {
            "_converted".into()
        } else {
            format!("_converted_{index}")
        };
        let candidate = directory.join(format!("{stem}{suffix}.{extension}"));
        if !candidate.exists() {
            return Ok(candidate);
        }
    }

    Err(ConvertError::Io(
        "Не удалось подобрать свободное имя результата".into(),
    ))
}

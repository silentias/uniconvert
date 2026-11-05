use crate::domain::ConvertError;
use std::path::{Path, PathBuf};

pub fn temporary_output(final_path: &Path) -> PathBuf {
    // TODO: Сначала сформировать соседний уникальный .tmp путь.
    // TODO: Затем учитывать коллизии и ограничения файловой системы.
    todo!("temporary output naming is not implemented")
}

pub fn commit(temp_path: &Path, final_path: &Path) -> Result<(), ConvertError> {
    // TODO: Сначала атомарно переименовать готовый файл.
    // TODO: Затем добавить безопасную замену и восстановление после сбоя.
    todo!("temporary output commit is not implemented")
}

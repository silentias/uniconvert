use crate::domain::ConvertError;
use std::path::Path;

pub fn validate_paths(input: &Path, output: &Path) -> Result<(), ConvertError> {
    // TODO: Сначала проверить существование входа и доступность выхода.
    // TODO: Затем оценить свободное место, права и конфликты имён.
    todo!("filesystem validation is not implemented")
}

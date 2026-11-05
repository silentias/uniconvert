use std::{error::Error, fmt};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConvertError {
    InvalidInput(String),
    UnsupportedFormat(String),
    IncompatibleCodec(String),
    ProcessFailed(String),
    Cancelled,
    Io(String),
}

impl fmt::Display for ConvertError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO: Сейчас вернуть понятное сообщение; затем добавить стабильные коды и локализацию.
        match self {
            Self::InvalidInput(message) => {
                write!(formatter, "Некорректный входной файл: {message}")
            }
            Self::UnsupportedFormat(format) => {
                write!(formatter, "Формат не поддерживается: {format}")
            }
            Self::IncompatibleCodec(message) => write!(formatter, "Несовместимый кодек: {message}"),
            Self::ProcessFailed(message) => write!(formatter, "Ошибка конвертации: {message}"),
            Self::Cancelled => formatter.write_str("Конвертация отменена"),
            Self::Io(message) => write!(formatter, "Ошибка файловой системы: {message}"),
        }
    }
}

impl Error for ConvertError {}

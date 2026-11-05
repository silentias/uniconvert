mod audio;
pub mod image;
mod subtitle;
mod video;

use crate::domain::{ConvertError, ConvertOutput, ConvertRequest, MediaKind};

pub use audio::AudioConverter;
pub use image::ImageConverter;
pub use subtitle::SubtitleConverter;
pub use video::VideoConverter;

pub trait Converter: Send + Sync {
    fn supports(&self, kind: MediaKind, request: &ConvertRequest) -> bool;
    fn convert(&self, request: &ConvertRequest) -> Result<ConvertOutput, ConvertError>;
}

pub fn registry() -> Vec<Box<dyn Converter>> {
    // TODO: Сначала зарегистрировать встроенные конвертеры.
    // TODO: Затем выбирать лучший конвертер по медиа и формату.
    // TODO: Позже поддержать внешние плагины и приоритеты.
    todo!("converter registry is not implemented")
}

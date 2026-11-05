mod audio_presets;
mod custom;
mod image_presets;
mod target_size;
mod video_presets;

use crate::domain::{ConvertError, ConvertOptions, MediaInfo, Preset};

pub use custom::CustomPreset;

pub trait PresetResolver: Send + Sync {
    fn descriptor(&self) -> Preset;
    fn resolve(&self, media: &MediaInfo) -> Result<ConvertOptions, ConvertError>;
}

pub fn registry() -> Vec<Box<dyn PresetResolver>> {
    // TODO: Сначала вернуть встроенные пресеты.
    // TODO: Затем фильтровать их по типу медиа и возможностям системы.
    // TODO: Позже объединить встроенные, пользовательские и плагины.
    todo!("preset registry is not implemented")
}

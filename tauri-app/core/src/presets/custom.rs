use super::PresetResolver;
use crate::domain::{ConvertError, ConvertOptions, MediaInfo, Preset};

pub struct CustomPreset {
    pub descriptor: Preset,
    pub options: ConvertOptions,
}

impl PresetResolver for CustomPreset {
    fn descriptor(&self) -> Preset {
        self.descriptor.clone()
    }
    fn resolve(&self, media: &MediaInfo) -> Result<ConvertOptions, ConvertError> {
        // TODO: Сначала проверить пользовательские параметры.
        // TODO: Затем мигрировать версии и исправлять несовместимые опции.
        todo!("custom preset resolution is not implemented")
    }
}

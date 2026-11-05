use super::PresetResolver;
use crate::domain::{ConvertError, ConvertOptions, MediaInfo, Preset};

pub struct ImagePreset;

impl PresetResolver for ImagePreset {
    fn descriptor(&self) -> Preset {
        // TODO: Сначала описать веб-, печатный и социальный пресеты.
        // TODO: Затем добавить профили конкретных сервисов.
        todo!("image preset descriptor is not implemented")
    }
    fn resolve(&self, media: &MediaInfo) -> Result<ConvertOptions, ConvertError> {
        // TODO: Сначала выбрать формат и качество.
        // TODO: Затем учитывать размеры, прозрачность и цветовой профиль.
        todo!("image preset resolution is not implemented")
    }
}

use super::PresetResolver;
use crate::domain::{ConvertError, ConvertOptions, MediaInfo, Preset};

pub struct AudioPreset;

impl PresetResolver for AudioPreset {
    fn descriptor(&self) -> Preset {
        // TODO: Сначала описать пресеты музыки, речи и подкаста.
        // TODO: Затем добавить платформенные профили.
        todo!("audio preset descriptor is not implemented")
    }
    fn resolve(&self, media: &MediaInfo) -> Result<ConvertOptions, ConvertError> {
        // TODO: Сначала выбрать кодек и битрейт.
        // TODO: Затем учитывать loudness, каналы и sample rate.
        todo!("audio preset resolution is not implemented")
    }
}

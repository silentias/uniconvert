use super::PresetResolver;
use crate::domain::{ConvertError, ConvertOptions, MediaInfo, Preset};

pub struct VideoPreset;

impl PresetResolver for VideoPreset {
    fn descriptor(&self) -> Preset {
        // TODO: Сначала описать базовый пресет для веба.
        // TODO: Затем добавить мобильные, архивные и платформенные варианты.
        todo!("video preset descriptor is not implemented")
    }
    fn resolve(&self, media: &MediaInfo) -> Result<ConvertOptions, ConvertError> {
        // TODO: Сначала развернуть качество в CRF и кодек.
        // TODO: Затем учитывать источник, фильтры и доступный GPU.
        todo!("video preset resolution is not implemented")
    }
}

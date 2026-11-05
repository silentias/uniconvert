use crate::domain::ConvertError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HardwareEncoder {
    Nvenc,
    VideoToolbox,
    Vaapi,
    Qsv,
    Amf,
}

pub fn detect() -> Result<Vec<HardwareEncoder>, ConvertError> {
    // TODO: Сначала обнаружить известные энкодеры через FFmpeg.
    // TODO: Затем проверить реальную работоспособность пробным запуском.
    todo!("hardware acceleration detection is not implemented")
}

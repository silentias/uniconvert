use crate::domain::{ConvertError, MediaInfo, TargetSize};

pub fn calculate_video_bitrate(
    media: &MediaInfo,
    target: TargetSize,
    audio_bitrate: u64,
) -> Result<u64, ConvertError> {
    // TODO: Сначала вычислить бюджет видео по длительности и размеру.
    // TODO: Затем учесть накладные расходы контейнера и переменный битрейт.
    // TODO: Позже оценивать ожидаемое качество и выдавать предупреждения.
    todo!("target-size bitrate calculation is not implemented")
}

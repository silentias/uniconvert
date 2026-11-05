use crate::domain::{ConvertError, ConvertOptions, MediaInfo};

pub fn audio_size(
    format: &str,
    duration: Option<f64>,
    tempo: f32,
    bitrate_kbps: Option<u32>,
    sample_rate: Option<u32>,
    channels: Option<u32>,
) -> Result<Option<u64>, ConvertError> {
    if !tempo.is_finite() || !(0.5..=2.0).contains(&tempo) {
        return Err(ConvertError::InvalidInput(
            "Темп должен быть от 0,5 до 2,0".into(),
        ));
    }
    let Some(duration) = duration.filter(|value| value.is_finite() && *value > 0.0) else {
        return Ok(None);
    };
    let duration = duration / tempo as f64;
    let bytes = match format.to_ascii_lowercase().as_str() {
        "wav" | "aiff" => match (sample_rate, channels) {
            (Some(rate), Some(channels)) if rate > 0 && channels > 0 => {
                duration * rate as f64 * channels as f64 * 2.0 + 4096.0
            }
            _ => return Ok(None),
        },
        "flac" => return Ok(None),
        _ => match bitrate_kbps.filter(|value| *value > 0) {
            Some(bitrate) => duration * bitrate as f64 * 1000.0 / 8.0 * 1.02 + 4096.0,
            None => return Ok(None),
        },
    };
    Ok(Some(bytes.ceil() as u64))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn calculates_pcm_and_bitrate_with_tempo_without_guessing_lossless_compression() {
        assert_eq!(
            audio_size("wav", Some(10.0), 1.0, None, Some(48000), Some(2)).unwrap(),
            Some(1924096)
        );
        assert_eq!(
            audio_size("aiff", Some(10.0), 2.0, None, Some(48000), Some(1)).unwrap(),
            Some(484096)
        );
        assert_eq!(
            audio_size("mp3", Some(10.0), 1.0, Some(192), None, None).unwrap(),
            Some(248896)
        );
        assert_eq!(
            audio_size("flac", Some(10.0), 1.0, None, None, None).unwrap(),
            None
        );
        assert_eq!(
            audio_size("mp3", Some(10.0), 1.0, None, None, None).unwrap(),
            None
        );
        assert!(audio_size("wav", Some(10.0), 0.0, None, None, None).is_err());
    }
}

pub fn estimate_size(media: &MediaInfo, options: &ConvertOptions) -> Result<u64, ConvertError> {
    // TODO: Сначала оценить размер по битрейту и длительности.
    // TODO: Затем учесть контейнер, VBR и свойства кодека.
    todo!("fast size estimation is not implemented")
}

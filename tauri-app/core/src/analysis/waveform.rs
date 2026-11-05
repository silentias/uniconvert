use crate::{
    domain::ConvertError,
    infra::{bin_resolver, process},
};
use std::{
    fs::File,
    io::{BufReader, Read, Seek, SeekFrom},
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
};

pub struct AudioPreview {
    pub duration_seconds: f64,
    pub peaks: Vec<f32>,
}

pub fn prepare_audio_preview<F: FnMut(f32)>(
    input: &Path,
    output: &Path,
    cancelled: &AtomicBool,
    on_progress: F,
) -> Result<AudioPreview, ConvertError> {
    let duration = super::probe::probe(input)
        .ok()
        .and_then(|info| info.duration_seconds);
    let ffmpeg = bin_resolver::resolve("ffmpeg")?;
    let args = vec![
        "-nostdin".into(),
        "-hide_banner".into(),
        "-loglevel".into(),
        "error".into(),
        "-i".into(),
        input.to_string_lossy().into_owned(),
        "-map".into(),
        "0:a:0".into(),
        "-vn".into(),
        "-map_metadata".into(),
        "-1".into(),
        "-c:a".into(),
        "pcm_s16le".into(),
        "-ar".into(),
        "44100".into(),
        "-ac".into(),
        "2".into(),
        "-rf64".into(),
        "never".into(),
        output.to_string_lossy().into_owned(),
    ];
    let result = (|| {
        let result = process::run_with_progress(
            &ffmpeg.to_string_lossy(),
            &args,
            duration,
            cancelled,
            on_progress,
        )?;
        if !result.status.success() {
            return Err(ConvertError::ProcessFailed(
                String::from_utf8_lossy(&result.stderr).trim().into(),
            ));
        }
        read_waveform(output, 1200, cancelled)
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(output);
    }
    result
}

pub fn generate_waveform(path: &Path, samples: usize) -> Result<Vec<f32>, ConvertError> {
    Ok(read_waveform(path, samples, &AtomicBool::new(false))?.peaks)
}

// Reads PCM16 WAV generated above, respecting metadata chunks and padding.
fn read_waveform(
    path: &Path,
    bins: usize,
    cancelled: &AtomicBool,
) -> Result<AudioPreview, ConvertError> {
    let invalid =
        || ConvertError::InvalidInput("Не удалось прочитать WAV для прослушивания".into());
    let io_error = |error: std::io::Error| ConvertError::Io(error.to_string());
    if bins == 0 || bins > 10_000 {
        return Err(invalid());
    }
    let file = File::open(path).map_err(io_error)?;
    let file_len = file.metadata().map_err(io_error)?.len();
    let mut reader = BufReader::new(file);
    let mut header = [0; 12];
    reader.read_exact(&mut header).map_err(io_error)?;
    if &header[..4] != b"RIFF" || &header[8..] != b"WAVE" {
        return Err(invalid());
    }
    let mut format = None;
    loop {
        let mut chunk = [0; 8];
        reader.read_exact(&mut chunk).map_err(io_error)?;
        let length = u32::from_le_bytes(chunk[4..].try_into().unwrap()) as u64;
        let offset = reader.stream_position().map_err(io_error)?;
        if offset + length > file_len {
            return Err(invalid());
        }
        if &chunk[..4] == b"fmt " {
            if length < 16 {
                return Err(invalid());
            }
            let mut bytes = [0; 16];
            reader.read_exact(&mut bytes).map_err(io_error)?;
            let channels = u16::from_le_bytes([bytes[2], bytes[3]]) as u64;
            let rate = u32::from_le_bytes(bytes[4..8].try_into().unwrap()) as u64;
            if u16::from_le_bytes([bytes[0], bytes[1]]) != 1
                || u16::from_le_bytes([bytes[14], bytes[15]]) != 16
                || channels == 0
                || rate == 0
            {
                return Err(invalid());
            }
            format = Some((channels, rate));
        } else if &chunk[..4] == b"data" {
            let (channels, rate) = format.ok_or_else(invalid)?;
            if length == 0 || length % (channels * 2) != 0 {
                return Err(invalid());
            }
            let count = length / 2;
            let mut peaks = vec![0.0_f32; bins];
            let mut buffer = [0_u8; 8192];
            let mut index = 0_u64;
            while index < count {
                if cancelled.load(Ordering::Relaxed) {
                    return Err(ConvertError::Cancelled);
                }
                let bytes = ((count - index) * 2).min(buffer.len() as u64) as usize;
                reader.read_exact(&mut buffer[..bytes]).map_err(io_error)?;
                for sample in buffer[..bytes].chunks_exact(2) {
                    let amplitude =
                        (i16::from_le_bytes([sample[0], sample[1]]) as f32).abs() / 32768.0;
                    let bin = (index * bins as u64 / count) as usize;
                    peaks[bin] = peaks[bin].max(amplitude);
                    index += 1;
                }
            }
            return Ok(AudioPreview {
                duration_seconds: count as f64 / channels as f64 / rate as f64,
                peaks,
            });
        }
        reader
            .seek(SeekFrom::Start(offset + length + length % 2))
            .map_err(io_error)?;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn reads_pcm_peaks_and_skips_padded_metadata() {
        let mut wav = tempfile::NamedTempFile::new().unwrap();
        let mut bytes = b"RIFF".to_vec();
        bytes.extend(54_u32.to_le_bytes());
        bytes.extend(b"WAVEJUNK");
        bytes.extend(1_u32.to_le_bytes());
        bytes.extend([42, 0]);
        bytes.extend(b"fmt ");
        bytes.extend(16_u32.to_le_bytes());
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend(2_u32.to_le_bytes());
        bytes.extend(4_u32.to_le_bytes());
        bytes.extend(2_u16.to_le_bytes());
        bytes.extend(16_u16.to_le_bytes());
        bytes.extend(b"data");
        bytes.extend(8_u32.to_le_bytes());
        for sample in [0_i16, i16::MIN, 16384, 0] {
            bytes.extend(sample.to_le_bytes());
        }
        wav.write_all(&bytes).unwrap();
        let preview = read_waveform(wav.path(), 2, &AtomicBool::new(false)).unwrap();
        assert_eq!(preview.duration_seconds, 2.0);
        assert_eq!(preview.peaks, vec![1.0, 0.5]);
        assert!(matches!(
            read_waveform(wav.path(), 2, &AtomicBool::new(true)),
            Err(ConvertError::Cancelled)
        ));
        assert!(read_waveform(wav.path(), 0, &AtomicBool::new(false)).is_err());
        wav.as_file().set_len(bytes.len() as u64 - 1).unwrap();
        assert!(read_waveform(wav.path(), 2, &AtomicBool::new(false)).is_err());
    }
}

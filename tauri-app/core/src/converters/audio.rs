use super::Converter;
use std::{sync::atomic::AtomicBool, time::Instant};

use crate::{
    domain::{Container, ConvertError, ConvertOutput, ConvertRequest, MediaKind},
    infra::{bin_resolver, process},
};

pub struct AudioConverter;

impl AudioConverter {
    pub fn convert_with_progress<F>(
        &self,
        request: &ConvertRequest,
        duration_seconds: Option<f64>,
        cancelled: &AtomicBool,
        on_progress: F,
    ) -> Result<ConvertOutput, ConvertError>
    where
        F: FnMut(f32),
    {
        self.convert_inner(request, duration_seconds, cancelled, on_progress)
    }

    fn convert_inner<F>(
        &self,
        request: &ConvertRequest,
        duration_seconds: Option<f64>,
        cancelled: &AtomicBool,
        mut on_progress: F,
    ) -> Result<ConvertOutput, ConvertError>
    where
        F: FnMut(f32),
    {
        if !is_audio_container(request.options.container) {
            return Err(ConvertError::UnsupportedFormat(
                request.options.container.extension().into(),
            ));
        }

        let started_at = Instant::now();
        let duration_seconds = crate::analysis::audio_trim::trimmed_duration(
            request.options.audio_trim_start,
            request.options.audio_trim_end,
            duration_seconds,
        )?
        .map(|duration| duration / request.options.audio_tempo.unwrap_or(1.0) as f64);
        let ffmpeg = bin_resolver::resolve("ffmpeg")?;
        let input_size = std::fs::metadata(&request.input)
            .map_err(|error| ConvertError::Io(error.to_string()))?
            .len();
        let parent = request
            .output
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
            .unwrap_or_else(|| std::path::Path::new("."));
        // Publish only a completed file, without ever overwriting an existing path.
        // TempPath removes partial output on cancellation and every error path.
        let temporary = tempfile::Builder::new()
            .prefix(".uniconvert-")
            .suffix(&format!(".{}", request.options.container.extension()))
            .tempfile_in(parent)
            .map_err(|error| ConvertError::Io(error.to_string()))?
            .into_temp_path();
        let mut args = audio_arguments(request);
        args.push(temporary.to_string_lossy().into_owned());
        let result = process::run_with_progress(
            &ffmpeg.to_string_lossy(),
            &args,
            duration_seconds,
            cancelled,
            &mut on_progress,
        );
        let result = result?;
        if !result.status.success() {
            return Err(ConvertError::ProcessFailed(
                String::from_utf8_lossy(&result.stderr).trim().to_owned(),
            ));
        }

        if cancelled.load(std::sync::atomic::Ordering::Relaxed) {
            return Err(ConvertError::Cancelled);
        }
        temporary
            .persist_noclobber(&request.output)
            .map_err(|error| ConvertError::Io(error.to_string()))?;
        let output_size = std::fs::metadata(&request.output)
            .map_err(|error| ConvertError::Io(error.to_string()))?
            .len();
        on_progress(100.0);
        Ok(ConvertOutput {
            path: request.output.clone(),
            input_size,
            output_size,
            elapsed_millis: started_at.elapsed().as_millis() as u64,
        })
    }
}

impl Converter for AudioConverter {
    fn supports(&self, kind: MediaKind, request: &ConvertRequest) -> bool {
        // TODO: Сначала распознавать аудиовходы и базовые форматы.
        // TODO: Затем проверять совместимость кодека и контейнера.
        kind == MediaKind::Audio && is_audio_container(request.options.container)
    }

    fn convert(&self, request: &ConvertRequest) -> Result<ConvertOutput, ConvertError> {
        // TODO: Сначала реализовать обычное перекодирование.
        // TODO: Затем добавить ресемплинг, каналы и нормализацию.
        let cancelled = AtomicBool::new(false);
        self.convert_inner(request, None, &cancelled, |_| {})
    }
}

fn audio_arguments(request: &ConvertRequest) -> Vec<String> {
    let mut args = vec![
        "-nostdin".into(),
        "-y".into(), // Only our private temporary output is passed to FFmpeg.
        "-hide_banner".into(),
        "-loglevel".into(),
        "error".into(),
    ];
    // Accurate input seeking avoids decoding the entire prefix.
    if let Some(start) = request.options.audio_trim_start {
        args.extend(["-ss".into(), start.to_string()]);
    }
    args.extend([
        "-i".into(),
        request.input.to_string_lossy().into_owned(),
        "-vn".into(),
        "-map".into(),
        "0:a:0".into(),
    ]);
    if let Some(bitrate) = request.options.audio_bitrate_kbps {
        args.extend(["-b:a".into(), format!("{bitrate}k")]);
    }
    if let Some(sample_rate) = request.options.audio_sample_rate_hz {
        args.extend(["-ar".into(), sample_rate.to_string()]);
    }
    if let Some(channels) = request.options.audio_channels {
        args.extend(["-ac".into(), channels.to_string()]);
    }

    let mut filters = Vec::new();
    if let Some(end) = request.options.audio_trim_end {
        let duration = end - request.options.audio_trim_start.unwrap_or(0.0);
        filters.push(format!("atrim=duration={duration}"));
    }
    if request.options.audio_trim_start.is_some() || request.options.audio_trim_end.is_some() {
        filters.push("asetpts=PTS-STARTPTS".into());
    }
    if let Some(volume) = request.options.audio_volume_db {
        if volume != 0.0 {
            filters.push(format!("volume={volume}dB"));
        }
    }
    let tempo = request.options.audio_tempo.unwrap_or(1.0);
    let pitch_semitones = request.options.audio_pitch_semitones.unwrap_or(0.0);
    if tempo != 1.0 || pitch_semitones != 0.0 {
        let pitch = 2_f32.powf(pitch_semitones / 12.0);
        filters.push(format!("rubberband=tempo={tempo}:pitch={pitch:.6}"));
    }
    if request.options.audio_normalize_lufs {
        let target = request.options.audio_target_lufs.unwrap_or(-14.0);
        filters.push(format!("loudnorm=I={target}:TP=-1.5:LRA=11"));
    }
    if !filters.is_empty() {
        args.extend(["-af".into(), filters.join(",")]);
    }
    if request.options.strip_metadata {
        args.extend(["-map_metadata".into(), "-1".into()]);
    }
    args
}

fn is_audio_container(container: Container) -> bool {
    matches!(
        container,
        Container::Mp3
            | Container::Wav
            | Container::Flac
            | Container::Ogg
            | Container::Aac
            | Container::Opus
            | Container::M4a
            | Container::Wma
            | Container::Aiff
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{CompressionMode, ConvertOptions};

    fn request(input: std::path::PathBuf, output: std::path::PathBuf) -> ConvertRequest {
        ConvertRequest {
            input,
            output,
            options: ConvertOptions {
                container: Container::Wav,
                video_codec: None,
                audio_codec: None,
                compression: CompressionMode::Custom,
                quality: None,
                strip_metadata: true,
                audio_bitrate_kbps: None,
                audio_sample_rate_hz: Some(8000),
                audio_channels: Some(1),
                audio_volume_db: None,
                audio_tempo: None,
                audio_pitch_semitones: None,
                audio_normalize_lufs: false,
                audio_target_lufs: None,
                audio_trim_start: Some(1.25),
                audio_trim_end: Some(2.5),
                video_crf: None,
                video_height: None,
                video_fps: None,
                copy_streams: false,
            },
        }
    }

    #[test]
    fn trims_before_loudness_processing_and_resets_timestamps() {
        let mut request = request("source.wav".into(), "result.wav".into());
        request.options.audio_volume_db = Some(-3.0);
        request.options.audio_tempo = Some(1.25);
        request.options.audio_pitch_semitones = Some(3.0);
        request.options.audio_normalize_lufs = true;
        let args = audio_arguments(&request);
        let seek = args.iter().position(|arg| arg == "-ss").unwrap();
        let input = args.iter().position(|arg| arg == "-i").unwrap();
        assert!(seek < input);
        let filter = args.iter().position(|arg| arg == "-af").unwrap();
        assert_eq!(
            args[filter + 1],
            "atrim=duration=1.25,asetpts=PTS-STARTPTS,volume=-3dB,rubberband=tempo=1.25:pitch=1.189207,loudnorm=I=-14:TP=-1.5:LRA=11"
        );
        request.options.audio_trim_start = None;
        request.options.audio_trim_end = None;
        let full = audio_arguments(&request);
        assert!(!full.iter().any(|arg| arg == "-ss" || arg.contains("atrim")));
    }

    #[test]
    #[ignore = "Requires FFmpeg; run with --include-ignored and UNICONVERT_FFMPEG_PATH"]
    fn converts_exact_selected_samples_and_prepares_preview() {
        let directory = tempfile::tempdir().unwrap();
        let input = directory.path().join("source.wav");
        let output = directory.path().join("result.wav");
        let ffmpeg = bin_resolver::resolve("ffmpeg").unwrap();
        let source = process::run(
            &ffmpeg.to_string_lossy(),
            &[
                "-nostdin".into(),
                "-f".into(),
                "lavfi".into(),
                "-i".into(),
                "aevalsrc=if(lt(t\\,1)\\,0.1\\,0.5):s=8000:d=3".into(),
                "-c:a".into(),
                "pcm_s16le".into(),
                input.to_string_lossy().into_owned(),
            ],
        )
        .unwrap();
        assert!(
            source.status.success(),
            "{}",
            String::from_utf8_lossy(&source.stderr)
        );
        let request = request(input.clone(), output.clone());
        let mut progress = Vec::new();
        AudioConverter
            .convert_with_progress(&request, Some(3.0), &AtomicBool::new(false), |value| {
                progress.push(value)
            })
            .unwrap();
        let decoded = process::run(
            &ffmpeg.to_string_lossy(),
            &[
                "-i".into(),
                output.to_string_lossy().into_owned(),
                "-f".into(),
                "s16le".into(),
                "-c:a".into(),
                "pcm_s16le".into(),
                "pipe:1".into(),
            ],
        )
        .unwrap();
        assert!(decoded.status.success());
        assert_eq!(decoded.stdout.len(), 20_000); // 1.25 seconds, 8000 Hz, mono PCM16.
        assert!(decoded.stdout.chunks_exact(2).all(|sample| {
            (i16::from_le_bytes([sample[0], sample[1]]) as i32 - 16384).abs() <= 1
        }));
        assert!(progress.last().is_some_and(|value| *value > 99.0));
        let preview_path = directory.path().join("preview.wav");
        let preview = crate::analysis::waveform::prepare_audio_preview(
            &input,
            &preview_path,
            &AtomicBool::new(false),
            |_| {},
        )
        .unwrap();
        assert!((preview.duration_seconds - 3.0).abs() < 0.001);
        assert_eq!(preview.peaks.len(), 1200);
        let cancelled_path = directory.path().join("cancelled.wav");
        assert!(matches!(
            crate::analysis::waveform::prepare_audio_preview(
                &input,
                &cancelled_path,
                &AtomicBool::new(true),
                |_| {}
            ),
            Err(ConvertError::Cancelled)
        ));
        assert!(!cancelled_path.exists());

        let preserved = std::fs::read(&output).unwrap();
        assert!(matches!(
            AudioConverter.convert_with_progress(
                &request,
                Some(3.0),
                &AtomicBool::new(false),
                |_| {}
            ),
            Err(ConvertError::Io(_))
        ));
        assert_eq!(std::fs::read(&output).unwrap(), preserved);

        let cancelled = AtomicBool::new(false);
        let mut cancelled_request = request.clone();
        cancelled_request.output = directory.path().join("cancelled-result.wav");
        assert!(matches!(
            AudioConverter
                .convert_with_progress(&cancelled_request, Some(3.0), &cancelled, |_| cancelled
                    .store(true, std::sync::atomic::Ordering::Relaxed)),
            Err(ConvertError::Cancelled)
        ));
        assert!(!cancelled_request.output.exists());

        let mut failed_request = request.clone();
        failed_request.input = directory.path().join("invalid.wav");
        std::fs::write(&failed_request.input, b"invalid audio").unwrap();
        failed_request.output = directory.path().join("failed-result.wav");
        assert!(matches!(
            AudioConverter.convert_with_progress(
                &failed_request,
                Some(3.0),
                &AtomicBool::new(false),
                |_| {}
            ),
            Err(ConvertError::ProcessFailed(_))
        ));
        assert!(!failed_request.output.exists());
        assert!(!std::fs::read_dir(directory.path())
            .unwrap()
            .any(|entry| entry
                .unwrap()
                .file_name()
                .to_string_lossy()
                .starts_with(".uniconvert-")));
    }
}

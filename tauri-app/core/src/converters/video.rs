use super::Converter;
use std::{sync::atomic::AtomicBool, time::Instant};

use crate::{
    domain::{
        AudioCodec, Container, ConvertError, ConvertOutput, ConvertRequest, MediaKind, VideoCodec,
    },
    infra::{bin_resolver, process},
};

pub struct VideoConverter;

impl VideoConverter {
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
        on_progress: F,
    ) -> Result<ConvertOutput, ConvertError>
    where
        F: FnMut(f32),
    {
        if !is_video_container(request.options.container) {
            return Err(ConvertError::UnsupportedFormat(
                request.options.container.extension().into(),
            ));
        }
        let started_at = Instant::now();
        let ffmpeg = bin_resolver::resolve("ffmpeg")?;
        let mut args = video_arguments(request);
        args.push(request.output.to_string_lossy().into_owned());
        let result = process::run_with_progress(
            &ffmpeg.to_string_lossy(),
            &args,
            duration_seconds,
            cancelled,
            on_progress,
        );
        if matches!(result, Err(ConvertError::Cancelled)) {
            let _ = std::fs::remove_file(&request.output);
        }
        let result = result?;
        if !result.status.success() {
            return Err(ConvertError::ProcessFailed(
                String::from_utf8_lossy(&result.stderr).trim().to_owned(),
            ));
        }
        let input_size = std::fs::metadata(&request.input)
            .map_err(|error| ConvertError::Io(error.to_string()))?
            .len();
        let output_size = std::fs::metadata(&request.output)
            .map_err(|error| ConvertError::Io(error.to_string()))?
            .len();
        Ok(ConvertOutput {
            path: request.output.clone(),
            input_size,
            output_size,
            elapsed_millis: started_at.elapsed().as_millis() as u64,
        })
    }
}

impl Converter for VideoConverter {
    fn supports(&self, kind: MediaKind, request: &ConvertRequest) -> bool {
        // TODO: Сначала принимать видео с базовыми контейнерами.
        // TODO: Затем учитывать кодеки, GPU и сложные pipeline.
        kind == MediaKind::Video && is_video_container(request.options.container)
    }

    fn convert(&self, request: &ConvertRequest) -> Result<ConvertOutput, ConvertError> {
        // TODO: Затем добавить проверку совместимости кодеков с контейнерами.
        // TODO: Затем подключить пресеты, точный прогресс и отмену.
        // TODO: Позже добавить two-pass и аппаратное ускорение.
        let cancelled = AtomicBool::new(false);
        self.convert_inner(request, None, &cancelled, |_| {})
    }
}

fn video_arguments(request: &ConvertRequest) -> Vec<String> {
    let mut args = vec![
        "-nostdin".into(),
        "-hide_banner".into(),
        "-loglevel".into(),
        "error".into(),
        "-i".into(),
        request.input.to_string_lossy().into_owned(),
    ];

    if request.options.copy_streams {
        args.extend(["-map".into(), "0".into(), "-c".into(), "copy".into()]);
    } else {
        if let Some(codec) = request.options.video_codec {
            args.extend(["-c:v".into(), video_encoder(codec).into()]);
        }
        if let Some(crf) = request.options.video_crf {
            args.extend(["-crf".into(), crf.to_string()]);
            if matches!(
                request.options.video_codec,
                Some(VideoCodec::Vp9 | VideoCodec::Av1)
            ) {
                args.extend(["-b:v".into(), "0".into()]);
            }
        }
        if let Some(height) = request.options.video_height {
            args.extend(["-vf".into(), format!("scale=-2:{height}")]);
        }
        if let Some(fps) = request.options.video_fps {
            args.extend(["-r".into(), fps.to_string()]);
        }
        if let Some(codec) = request.options.audio_codec {
            args.extend(["-c:a".into(), audio_encoder(codec).into()]);
        }
        if let Some(bitrate) = request.options.audio_bitrate_kbps {
            args.extend(["-b:a".into(), format!("{bitrate}k")]);
        }
    }
    if request.options.strip_metadata {
        args.extend(["-map_metadata".into(), "-1".into()]);
    }
    args
}

fn video_encoder(codec: VideoCodec) -> &'static str {
    match codec {
        VideoCodec::H264 => "libx264",
        VideoCodec::H265 => "libx265",
        VideoCodec::Vp9 => "libvpx-vp9",
        VideoCodec::Av1 => "libaom-av1",
        VideoCodec::Copy => "copy",
    }
}

fn audio_encoder(codec: AudioCodec) -> &'static str {
    match codec {
        AudioCodec::Aac => "aac",
        AudioCodec::Mp3 => "libmp3lame",
        AudioCodec::Opus => "libopus",
        AudioCodec::Flac => "flac",
        AudioCodec::Pcm => "pcm_s16le",
        AudioCodec::Copy => "copy",
    }
}

fn is_video_container(container: Container) -> bool {
    matches!(
        container,
        Container::Mp4 | Container::Mkv | Container::Webm | Container::Mov | Container::Avi
    )
}

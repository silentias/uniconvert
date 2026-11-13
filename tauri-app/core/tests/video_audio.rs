use std::{path::Path, sync::atomic::AtomicBool};
use uniconvert_core::{
    analysis::{metadata, probe},
    audio::{prepare_conversion, prepare_conversion_from_media, AudioOptions},
    converters::AudioConverter,
    infra::{bin_resolver, process},
};

fn ffmpeg(args: &[&str], output: &Path) {
    let binary = bin_resolver::resolve("ffmpeg").unwrap();
    let mut args: Vec<String> = args.iter().map(|value| value.to_string()).collect();
    args.push(output.to_string_lossy().into_owned());
    let result = process::run(&binary.to_string_lossy(), &args).unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
}

#[test]
#[ignore = "Requires FFmpeg/FFprobe"]
fn extracts_audio_from_video_and_rejects_silent_video() {
    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("movie.mp4");
    ffmpeg(
        &[
            "-y",
            "-f",
            "lavfi",
            "-i",
            "color=size=32x32:rate=10:duration=2",
            "-f",
            "lavfi",
            "-i",
            "sine=frequency=440:duration=2",
            "-c:v",
            "mpeg4",
            "-c:a",
            "aac",
            "-shortest",
        ],
        &input,
    );
    let cancelled = AtomicBool::new(false);
    let options = AudioOptions {
        format: "wav".into(),
        sample_rate_hz: Some(8000),
        channels: Some(1),
        ..Default::default()
    };
    assert!(prepare_conversion(&input, None, &options, &cancelled).is_err());
    let prepared = prepare_conversion_from_media(&input, None, &options, &cancelled, true).unwrap();
    let result = AudioConverter
        .convert_with_progress(
            &prepared.request,
            prepared.info.duration_seconds,
            &cancelled,
            |_| {},
        )
        .unwrap();
    let info = probe::probe_audio(&result.path, &cancelled).unwrap();
    assert_eq!(info.sample_rate_hz, Some(8000));
    assert_eq!(info.channels, Some(1));
    assert!((info.duration_seconds.unwrap() - 2.0).abs() < 0.15);
    assert!(metadata::extract(&result.path).unwrap().video.is_none());
    let silent = directory.path().join("silent.mp4");
    ffmpeg(
        &[
            "-y",
            "-i",
            input.to_str().unwrap(),
            "-map",
            "0:v:0",
            "-c:v",
            "copy",
            "-an",
        ],
        &silent,
    );
    let error = prepare_conversion_from_media(&silent, None, &options, &cancelled, true)
        .err()
        .unwrap();
    assert!(format!("{error:?}").contains("аудиопотоки"));
}

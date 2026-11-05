use serde_json::Value;
use std::{
    path::Path,
    process::{Command, Output},
};
use uniconvert_core::infra::bin_resolver;

fn cli(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_uniconvert"))
        .args(args)
        .output()
        .unwrap()
}

fn json(output: &Output) -> Value {
    assert!(
        output.stderr.is_empty(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("one JSON result in stdout")
}

#[test]
fn help_and_json_errors_do_not_require_media_tools() {
    let help = cli(&["--help"]);
    assert!(help.status.success());
    assert!(String::from_utf8_lossy(&help.stdout).contains("convert"));
    let missing_to = cli(&["convert", "source.wav", "--json"]);
    assert_eq!(missing_to.status.code(), Some(2));
    assert_eq!(json(&missing_to)["error"]["code"], "invalid_arguments");
    let invalid_format = cli(&["convert", "source.wav", "--to", "mp4", "--json"]);
    assert_eq!(invalid_format.status.code(), Some(2));
    assert_eq!(json(&invalid_format)["error"]["code"], "unsupported_format");
    let invalid_range = cli(&[
        "convert",
        "source.wav",
        "--to",
        "mp3",
        "--start",
        "5",
        "--end",
        "4",
        "--json",
    ]);
    assert_eq!(invalid_range.status.code(), Some(2));
    assert_eq!(json(&invalid_range)["error"]["code"], "invalid_input");
    for option in [["--tempo", "2.1"], ["--pitch", "13"]] {
        let invalid_effect = cli(&[
            "convert",
            "source.wav",
            "--to",
            "mp3",
            option[0],
            option[1],
            "--json",
        ]);
        assert_eq!(invalid_effect.status.code(), Some(2));
        assert_eq!(json(&invalid_effect)["error"]["code"], "invalid_input");
    }
    let no_file = cli(&["probe", "definitely-missing-audio-file.wav", "--json"]);
    assert_eq!(no_file.status.code(), Some(2));
    assert_eq!(json(&no_file)["error"]["code"], "invalid_input");
}

fn ffmpeg(args: &[&str]) -> Output {
    let output = Command::new(bin_resolver::resolve("ffmpeg").unwrap())
        .args(["-nostdin", "-hide_banner", "-loglevel", "error"])
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

#[test]
#[ignore = "Requires FFmpeg and FFprobe; run with --include-ignored"]
fn audio_cli_converts_probes_trims_and_preserves_files() {
    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("голос с пробелами.wav");
    ffmpeg(&[
        "-f",
        "lavfi",
        "-i",
        "aevalsrc=if(lt(t\\,1)\\,0.1\\,0.5):s=44100:d=3",
        "-c:a",
        "pcm_s16le",
        input.to_str().unwrap(),
    ]);
    let original = std::fs::read(&input).unwrap();
    let probe = cli(&["probe", input.to_str().unwrap(), "--json"]);
    assert!(probe.status.success());
    let info = json(&probe);
    assert_eq!(info["result"]["sample_rate_hz"], 44100);
    assert_eq!(info["result"]["channels"], 1);

    let converted = cli(&[
        "convert",
        input.to_str().unwrap(),
        "--to",
        "wav",
        "--start",
        "00:01.250",
        "--end",
        "2.5",
        "--sample-rate",
        "8000",
        "--json",
    ]);
    assert!(
        converted.status.success(),
        "{}",
        String::from_utf8_lossy(&converted.stdout)
    );
    let result = json(&converted);
    let path = result["result"]["path"].as_str().unwrap();
    assert_ne!(Path::new(path), input);
    let raw = ffmpeg(&["-i", path, "-f", "s16le", "-c:a", "pcm_s16le", "pipe:1"]);
    assert_eq!(raw.stdout.len(), 20_000);
    assert!(raw
        .stdout
        .chunks_exact(2)
        .all(|bytes| (i16::from_le_bytes([bytes[0], bytes[1]]) as i32 - 16384).abs() <= 1));
    assert_eq!(std::fs::read(&input).unwrap(), original);

    let transformed = cli(&[
        "convert",
        input.to_str().unwrap(),
        "--to",
        "wav",
        "--tempo",
        "2",
        "--output-dir",
        directory.path().to_str().unwrap(),
        "--json",
    ]);
    assert!(
        transformed.status.success(),
        "{}",
        String::from_utf8_lossy(&transformed.stderr)
    );
    let transformed_path = json(&transformed)["result"]["path"]
        .as_str()
        .unwrap()
        .to_owned();
    let transformed_info = cli(&["probe", &transformed_path, "--json"]);
    let transformed_duration = json(&transformed_info)["result"]["duration_seconds"]
        .as_f64()
        .unwrap();
    assert!((1.4..=1.6).contains(&transformed_duration));

    let pitch_shifted = cli(&[
        "convert",
        input.to_str().unwrap(),
        "--to",
        "wav",
        "--pitch",
        "12",
        "--output-dir",
        directory.path().to_str().unwrap(),
        "--json",
    ]);
    assert!(pitch_shifted.status.success());
    let pitch_shifted_path = json(&pitch_shifted)["result"]["path"]
        .as_str()
        .unwrap()
        .to_owned();
    let pitch_shifted_info = cli(&["probe", &pitch_shifted_path, "--json"]);
    let pitch_shifted_duration = json(&pitch_shifted_info)["result"]["duration_seconds"]
        .as_f64()
        .unwrap();
    assert!((2.9..=3.1).contains(&pitch_shifted_duration));

    let saved = std::fs::read(path).unwrap();
    let second = cli(&["convert", input.to_str().unwrap(), "--to", "wav", "--json"]);
    assert!(second.status.success());
    assert_ne!(json(&second)["result"]["path"], result["result"]["path"]);
    assert_eq!(std::fs::read(path).unwrap(), saved);

    let outputs = directory.path().join("new folder");
    for format in ["mp3", "flac", "aac", "ogg", "opus", "m4a", "wma", "aiff"] {
        let output = cli(&[
            "convert",
            input.to_str().unwrap(),
            "--to",
            format,
            "--output-dir",
            outputs.to_str().unwrap(),
            "--json",
        ]);
        assert!(
            output.status.success(),
            "{format}: {}",
            String::from_utf8_lossy(&output.stdout)
        );
        let output_path = json(&output)["result"]["path"].as_str().unwrap().to_owned();
        let info = cli(&["probe", &output_path, "--json"]);
        assert!(
            info.status.success(),
            "{format}: {}",
            String::from_utf8_lossy(&info.stdout)
        );
        assert!(json(&info)["result"]["duration_seconds"].as_f64().unwrap() > 2.8);
    }
    let invalid = cli(&[
        "convert",
        input.to_str().unwrap(),
        "--to",
        "mp3",
        "--end",
        "4",
        "--json",
    ]);
    assert_eq!(invalid.status.code(), Some(2));
    let video = directory.path().join("video.mp4");
    ffmpeg(&[
        "-f",
        "lavfi",
        "-i",
        "color=s=16x16:d=1",
        "-f",
        "lavfi",
        "-i",
        "sine=d=1",
        "-c:v",
        "mpeg4",
        "-c:a",
        "aac",
        video.to_str().unwrap(),
    ]);
    for args in [
        vec!["probe", video.to_str().unwrap(), "--json"],
        vec!["convert", video.to_str().unwrap(), "--to", "mp3", "--json"],
    ] {
        let output = cli(&args);
        assert_eq!(output.status.code(), Some(2));
        assert_eq!(json(&output)["error"]["code"], "invalid_input");
    }
    for folder in [directory.path(), outputs.as_path()] {
        assert!(!std::fs::read_dir(folder).unwrap().any(|entry| entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with(".uniconvert-")));
    }
}

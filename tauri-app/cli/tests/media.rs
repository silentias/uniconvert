use serde_json::{json, Value};
use std::process::{Command, Output};
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

fn one_pixel_bmp() -> Vec<u8> {
    vec![
        0x42, 0x4d, 58, 0, 0, 0, 0, 0, 0, 0, 54, 0, 0, 0, 40, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1,
        0, 24, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xff,
        0x40, 0x20, 0,
    ]
}

#[test]
fn image_and_gerber_commands_work_without_ffmpeg() {
    let directory = tempfile::tempdir().unwrap();
    let image = directory.path().join("pixel.bmp");
    std::fs::write(&image, one_pixel_bmp()).unwrap();

    let converted = cli(&[
        "image",
        image.to_str().unwrap(),
        "--to",
        "png",
        "--quality",
        "90",
        "--json",
    ]);
    assert!(
        converted.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&converted.stdout),
        String::from_utf8_lossy(&converted.stderr)
    );
    let output_path = json(&converted)["result"]["path"]
        .as_str()
        .unwrap()
        .to_owned();
    assert!(std::fs::read(output_path)
        .unwrap()
        .starts_with(b"\x89PNG\r\n\x1a\n"));

    let image_metadata = cli(&["image-meta", image.to_str().unwrap(), "--json"]);
    assert!(image_metadata.status.success());
    let image_metadata = json(&image_metadata);
    assert_eq!(image_metadata["result"]["format"], "BMP");
    assert_eq!(image_metadata["result"]["width"], 1);
    assert_eq!(image_metadata["result"]["height"], 1);
    assert_eq!(image_metadata["result"]["channelCount"], 3);
    assert_eq!(image_metadata["result"]["hasAlpha"], false);
    assert_eq!(image_metadata["result"]["exif"], json!([]));

    let gerber = directory.path().join("top copper.gtl");
    std::fs::write(
        &gerber,
        "%FSLAX46Y46*%\n%MOMM*%\n%TF.FileFunction,Copper,L1,Top,Signal*%\n%TF.GenerationSoftware,KiCad,Pcbnew,9.0*%\n%TF.CreationDate,2026-10-14T10:15:00+03:00*%\n%ADD10C,0.2*%\n%ADD10C,0.3*%\n%ADD12R,0.4X0.5*%\nD10*\nX0Y0D02*\nX100Y100D01*\nD11*\nX200Y200D03*\nM02*",
    )
    .unwrap();
    let metadata = cli(&["gerber-meta", gerber.to_str().unwrap(), "--json"]);
    assert!(metadata.status.success());
    let metadata = json(&metadata);
    assert_eq!(metadata["result"]["cadProgram"], "KiCad · Pcbnew · 9.0");
    assert_eq!(
        metadata["result"]["creationDate"],
        "2026-10-14T10:15:00+03:00"
    );
    assert_eq!(metadata["result"]["layerType"], "Copper");
    assert_eq!(metadata["result"]["side"], "top");
    assert_eq!(metadata["result"]["units"], "mm");
    assert_eq!(metadata["result"]["coordinateFormat"]["xIntegerDigits"], 4);
    assert_eq!(metadata["result"]["coordinateFormat"]["xDecimalDigits"], 6);
    assert_eq!(metadata["result"]["statistics"]["lineCount"], 1);
    assert_eq!(metadata["result"]["statistics"]["flashCount"], 1);
    assert_eq!(metadata["result"]["statistics"]["apertureCount"], 3);
    assert_eq!(
        metadata["result"]["statistics"]["usedDCodes"],
        json!([10, 11])
    );
    assert_eq!(metadata["result"]["apertures"][0]["shape"], "circle");
    assert_eq!(metadata["result"]["apertures"][0]["usageCount"], 1);
    assert_eq!(
        metadata["result"]["dCodeValidation"]["undefinedUsed"],
        json!([11])
    );
    assert_eq!(
        metadata["result"]["dCodeValidation"]["unusedDefined"],
        json!([12])
    );
    assert_eq!(
        metadata["result"]["dCodeValidation"]["duplicateDefinitions"],
        json!([10])
    );
    assert_eq!(metadata["result"]["structureValidation"]["isValid"], true);
    assert_eq!(
        metadata["result"]["structureValidation"]["issues"],
        json!([])
    );
}

#[test]
#[ignore = "Requires FFmpeg and FFprobe; run with --include-ignored"]
fn video_command_converts_and_reports_progress_result() {
    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("source video.mkv");
    let fixture = Command::new(bin_resolver::resolve("ffmpeg").unwrap())
        .args(["-nostdin", "-hide_banner", "-loglevel", "error"])
        .args([
            "-f",
            "lavfi",
            "-i",
            "testsrc2=size=64x64:rate=24:duration=1",
            "-f",
            "lavfi",
            "-i",
            "sine=frequency=440:duration=1",
            "-c:v",
            "mpeg4",
            "-c:a",
            "aac",
            input.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(fixture.status.success());

    let converted = cli(&[
        "video",
        input.to_str().unwrap(),
        "--to",
        "mp4",
        "--video-codec",
        "h264",
        "--crf",
        "28",
        "--height",
        "480",
        "--fps",
        "24",
        "--audio-codec",
        "aac",
        "--audio-bitrate",
        "96",
        "--json",
    ]);
    assert!(
        converted.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&converted.stdout),
        String::from_utf8_lossy(&converted.stderr)
    );
    let result = json(&converted);
    let output = result["result"]["path"].as_str().unwrap();
    assert!(std::fs::metadata(output).unwrap().len() > 0);
    assert_ne!(output, input.to_str().unwrap());
}

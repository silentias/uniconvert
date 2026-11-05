mod args;

use args::{Action, Cli};
use clap::Parser;
use serde_json::{json, Value};
use std::{
    io::{self, IsTerminal, Write},
    path::{Path, PathBuf},
    process::ExitCode,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
use uniconvert_core::{
    analysis::{gerber, image_metadata, probe},
    audio::prepare_conversion,
    converters::{AudioConverter, Converter, ImageConverter, VideoConverter},
    domain::{
        AudioCodec, CompressionMode, Container, ConvertError, ConvertOptions, ConvertRequest,
        MediaKind, VideoCodec,
    },
    infra::paths::available_output_path,
};

fn main() -> ExitCode {
    let arguments: Vec<_> = std::env::args_os().collect();
    // Honor JSON even on parser failures, but do not interpret a filename after --.
    let json_requested = arguments
        .iter()
        .skip(1)
        .take_while(|arg| *arg != "--")
        .any(|arg| arg == "--json");
    let cli = match Cli::try_parse_from(&arguments) {
        Ok(cli) => cli,
        Err(error) => {
            let code = error.exit_code() as u8;
            if code == 0 {
                let _ = error.print(); // --help and --version remain plain text.
            } else if json_requested {
                let _ = print_json(
                    &json!({"ok":false,"error":{"code":"invalid_arguments","message":error.to_string()}}),
                );
            } else {
                let _ = error.print();
            }
            return ExitCode::from(code);
        }
    };
    let cancelled = Arc::new(AtomicBool::new(false));
    let signal = cancelled.clone();
    if let Err(error) = ctrlc::set_handler(move || signal.store(true, Ordering::Relaxed)) {
        return report_error(
            &ConvertError::ProcessFailed(format!("Не удалось подключить Ctrl+C: {error}")),
            cli.json,
        );
    }
    match execute(&cli, &cancelled) {
        Ok(value) => {
            let result = if cli.json {
                print_json(&json!({"ok":true,"result":value}))
            } else {
                print_human(&cli.command, &value)
            };
            if result.is_err() {
                return ExitCode::from(1);
            }
            ExitCode::SUCCESS
        }
        Err(error) => report_error(&error, cli.json),
    }
}

fn execute(cli: &Cli, cancelled: &AtomicBool) -> Result<Value, ConvertError> {
    match &cli.command {
        Action::Probe { input } => Ok(json!(probe::probe_audio(input, cancelled)?)),
        Action::GerberMeta { input } => Ok(json!(gerber::extract_metadata(input)?)),
        Action::ImageMeta { input } => Ok(json!(image_metadata::extract_metadata(input)?)),
        Action::Audio(args) => {
            let show_progress = !cli.json && !cli.quiet && io::stderr().is_terminal();
            if show_progress {
                let _ = writeln!(io::stderr(), "Анализ аудио…");
            }
            let prepared = prepare_conversion(
                &args.input,
                args.output_dir.as_deref(),
                &args.options(),
                cancelled,
            )?;
            if show_progress {
                let _ = writeln!(io::stderr(), "Конвертация… (Ctrl+C — отмена)");
            }
            let mut last_percent = -1;
            let result = AudioConverter.convert_with_progress(
                &prepared.request,
                prepared.info.duration_seconds,
                cancelled,
                |percent| {
                    let percent = percent.floor() as i32;
                    if show_progress && percent != last_percent {
                        last_percent = percent;
                        let _ = write!(io::stderr(), "\rКонвертация: {percent:3}%");
                        let _ = io::stderr().flush();
                    }
                },
            );
            if show_progress {
                if result.is_ok() {
                    let _ = writeln!(io::stderr(), "\rКонвертация: 100%");
                } else {
                    let _ = writeln!(io::stderr());
                }
            }
            Ok(json!(result?))
        }
        Action::Image(args) => {
            let container = Container::from_image_extension(&args.format)
                .ok_or_else(|| ConvertError::UnsupportedFormat(args.format.clone()))?;
            let input = local_input(&args.input)?;
            let output = output_path(&input, args.output_dir.as_deref(), container)?;
            let request = ConvertRequest {
                input,
                output,
                options: ConvertOptions {
                    quality: Some(args.quality),
                    strip_metadata: args.strip_metadata,
                    ..base_options(container)
                },
            };
            let result = ImageConverter.convert(&request);
            if result.is_err() {
                let _ = std::fs::remove_file(&request.output);
            }
            Ok(json!(result?))
        }
        Action::Video(args) => {
            let container = video_container(&args.format)?;
            let input = local_input(&args.input)?;
            let media = probe::probe(&input)?;
            if media.kind != MediaKind::Video {
                return Err(ConvertError::InvalidInput(
                    "Ожидается видеофайл с видеопотоком".into(),
                ));
            }
            let output = output_path(&input, args.output_dir.as_deref(), container)?;
            let (video_codec, audio_codec, audio_bitrate_kbps) = if args.copy_streams {
                (Some(VideoCodec::Copy), Some(AudioCodec::Copy), None)
            } else {
                (
                    Some(parse_video_codec(&args.video_codec)?),
                    Some(parse_audio_codec(&args.audio_codec)?),
                    (args.audio_codec.to_ascii_lowercase() != "flac").then_some(args.audio_bitrate),
                )
            };
            let request = ConvertRequest {
                input,
                output,
                options: ConvertOptions {
                    video_codec,
                    audio_codec,
                    strip_metadata: args.strip_metadata,
                    audio_bitrate_kbps,
                    video_crf: (!args.copy_streams).then_some(args.crf),
                    video_height: (!args.copy_streams).then_some(args.height).flatten(),
                    video_fps: (!args.copy_streams).then_some(args.fps).flatten(),
                    copy_streams: args.copy_streams,
                    ..base_options(container)
                },
            };
            let show_progress = !cli.json && !cli.quiet && io::stderr().is_terminal();
            if show_progress {
                let _ = writeln!(io::stderr(), "Конвертация видео… (Ctrl+C — отмена)");
            }
            let mut last_percent = -1;
            let result = VideoConverter.convert_with_progress(
                &request,
                media.duration_seconds,
                cancelled,
                |percent| {
                    let percent = percent.floor() as i32;
                    if show_progress && percent != last_percent {
                        last_percent = percent;
                        let _ = write!(io::stderr(), "\rКонвертация: {percent:3}%");
                        let _ = io::stderr().flush();
                    }
                },
            );
            if result.is_err() {
                let _ = std::fs::remove_file(&request.output);
            }
            if show_progress {
                if result.is_ok() {
                    let _ = writeln!(io::stderr(), "\rКонвертация: 100%");
                } else {
                    let _ = writeln!(io::stderr());
                }
            }
            Ok(json!(result?))
        }
    }
}

fn local_input(input: &Path) -> Result<PathBuf, ConvertError> {
    let metadata = std::fs::metadata(input)
        .map_err(|error| ConvertError::InvalidInput(format!("{}: {error}", input.display())))?;
    if !metadata.is_file() {
        return Err(ConvertError::InvalidInput(
            "Ожидается локальный файл".into(),
        ));
    }
    std::fs::canonicalize(input).map_err(|error| ConvertError::Io(error.to_string()))
}

fn output_path(
    input: &Path,
    output_directory: Option<&Path>,
    container: Container,
) -> Result<PathBuf, ConvertError> {
    let directory = output_directory
        .map(PathBuf::from)
        .or_else(|| input.parent().map(PathBuf::from))
        .ok_or_else(|| {
            ConvertError::InvalidInput("Не удалось определить папку результата".into())
        })?;
    std::fs::create_dir_all(&directory).map_err(|error| ConvertError::Io(error.to_string()))?;
    let directory =
        std::fs::canonicalize(directory).map_err(|error| ConvertError::Io(error.to_string()))?;
    available_output_path(input, &directory, container)
}

fn base_options(container: Container) -> ConvertOptions {
    ConvertOptions {
        container,
        video_codec: None,
        audio_codec: None,
        compression: CompressionMode::Custom,
        quality: None,
        strip_metadata: false,
        audio_bitrate_kbps: None,
        audio_sample_rate_hz: None,
        audio_channels: None,
        audio_volume_db: None,
        audio_tempo: None,
        audio_pitch_semitones: None,
        audio_normalize_lufs: false,
        audio_target_lufs: None,
        audio_trim_start: None,
        audio_trim_end: None,
        video_crf: None,
        video_height: None,
        video_fps: None,
        copy_streams: false,
    }
}

fn video_container(format: &str) -> Result<Container, ConvertError> {
    match format.to_ascii_lowercase().as_str() {
        "mp4" => Ok(Container::Mp4),
        "mkv" => Ok(Container::Mkv),
        "webm" => Ok(Container::Webm),
        "mov" => Ok(Container::Mov),
        "avi" => Ok(Container::Avi),
        _ => Err(ConvertError::UnsupportedFormat(format.into())),
    }
}

fn parse_video_codec(codec: &str) -> Result<VideoCodec, ConvertError> {
    match codec.to_ascii_lowercase().as_str() {
        "h264" => Ok(VideoCodec::H264),
        "h265" => Ok(VideoCodec::H265),
        "vp9" => Ok(VideoCodec::Vp9),
        "av1" => Ok(VideoCodec::Av1),
        _ => Err(ConvertError::InvalidInput("Неизвестный видеокодек".into())),
    }
}

fn parse_audio_codec(codec: &str) -> Result<AudioCodec, ConvertError> {
    match codec.to_ascii_lowercase().as_str() {
        "aac" => Ok(AudioCodec::Aac),
        "mp3" => Ok(AudioCodec::Mp3),
        "opus" => Ok(AudioCodec::Opus),
        "flac" => Ok(AudioCodec::Flac),
        _ => Err(ConvertError::InvalidInput("Неизвестный аудиокодек".into())),
    }
}

fn print_json(value: &Value) -> io::Result<()> {
    let mut stdout = io::stdout().lock();
    serde_json::to_writer(&mut stdout, value)?;
    writeln!(stdout)
}

fn print_human(action: &Action, value: &Value) -> io::Result<()> {
    let mut stdout = io::stdout().lock();
    match action {
        Action::Audio(_) | Action::Image(_) | Action::Video(_) => writeln!(
            stdout,
            "Готово: {}\nРазмер: {} байт",
            value["path"].as_str().unwrap_or(""),
            value["output_size"]
        ),
        Action::Probe { .. } => {
            for (label, key, unit) in [
                ("Формат", "format", ""),
                ("Кодек", "codec", ""),
                ("Длительность", "duration_seconds", " с"),
                ("Размер", "file_size", " байт"),
                ("Битрейт", "bitrate", " бит/с"),
                ("Частота", "sample_rate_hz", " Гц"),
                ("Каналы", "channels", ""),
                ("Схема каналов", "channel_layout", ""),
            ] {
                let text = match &value[key] {
                    Value::Null => "неизвестно".into(),
                    Value::String(text) => text.clone(),
                    other => format!("{other}{unit}"),
                };
                writeln!(stdout, "{label}: {text}")?;
            }
            Ok(())
        }
        Action::GerberMeta { .. } => {
            writeln!(
                stdout,
                "CAD-программа: {}",
                value["cadProgram"].as_str().unwrap_or("не указана")
            )?;
            writeln!(
                stdout,
                "Дата создания: {}",
                value["creationDate"].as_str().unwrap_or("не указана")
            )?;
            writeln!(
                stdout,
                "Тип слоя: {}",
                value["layerType"].as_str().unwrap_or("не указан")
            )?;
            writeln!(
                stdout,
                "Сторона: {}",
                value["side"].as_str().unwrap_or("не указана")
            )?;
            writeln!(
                stdout,
                "Единицы: {}",
                value["units"].as_str().unwrap_or("не указаны")
            )?;
            let coordinate_format = &value["coordinateFormat"];
            if coordinate_format.is_null() {
                writeln!(stdout, "Формат координат: не указан")?;
            } else {
                writeln!(
                    stdout,
                    "Формат координат: X {}.{}, Y {}.{}; {}; {}",
                    coordinate_format["xIntegerDigits"],
                    coordinate_format["xDecimalDigits"],
                    coordinate_format["yIntegerDigits"],
                    coordinate_format["yDecimalDigits"],
                    coordinate_format["notation"]
                        .as_str()
                        .unwrap_or("notation unknown"),
                    coordinate_format["zeroSuppression"]
                        .as_str()
                        .unwrap_or("zero suppression unknown")
                )?;
            }
            let statistics = &value["statistics"];
            writeln!(stdout, "Линии: {}", statistics["lineCount"])?;
            writeln!(stdout, "Дуги: {}", statistics["arcCount"])?;
            writeln!(stdout, "Вспышки: {}", statistics["flashCount"])?;
            writeln!(stdout, "Области: {}", statistics["regionCount"])?;
            writeln!(stdout, "Апертуры: {}", statistics["apertureCount"])?;
            writeln!(
                stdout,
                "Макросы апертур: {}",
                statistics["apertureMacroCount"]
            )?;
            let format_d_codes = |value: &Value| {
                value
                    .as_array()
                    .map(|codes| {
                        codes
                            .iter()
                            .filter_map(Value::as_u64)
                            .map(|code| format!("D{code}"))
                            .collect::<Vec<_>>()
                            .join(", ")
                    })
                    .filter(|codes| !codes.is_empty())
                    .unwrap_or_else(|| "нет".into())
            };
            writeln!(
                stdout,
                "Используемые D-коды: {}",
                format_d_codes(&statistics["usedDCodes"])
            )?;
            let validation = &value["dCodeValidation"];
            writeln!(
                stdout,
                "Без определения: {}",
                format_d_codes(&validation["undefinedUsed"])
            )?;
            writeln!(
                stdout,
                "Не используются: {}",
                format_d_codes(&validation["unusedDefined"])
            )?;
            writeln!(
                stdout,
                "Повторные определения: {}",
                format_d_codes(&validation["duplicateDefinitions"])
            )?;
            let structure = &value["structureValidation"];
            writeln!(
                stdout,
                "Структура: {}",
                if structure["isValid"].as_bool().unwrap_or(false) {
                    "корректна"
                } else {
                    "есть ошибки"
                }
            )?;
            for issue in structure["issues"].as_array().into_iter().flatten() {
                let severity = match issue["severity"].as_str() {
                    Some("error") => "ошибка",
                    _ => "предупреждение",
                };
                let command = issue["command"]
                    .as_str()
                    .map(|command| format!(" [{command}]"))
                    .unwrap_or_default();
                writeln!(
                    stdout,
                    "  {severity}: {}{command}",
                    issue["message"].as_str().unwrap_or("")
                )?;
            }
            writeln!(stdout, "Список апертур:")?;
            for aperture in value["apertures"].as_array().into_iter().flatten() {
                let modifiers = aperture["modifiers"]
                    .as_array()
                    .map(|values| {
                        values
                            .iter()
                            .filter_map(Value::as_str)
                            .collect::<Vec<_>>()
                            .join(" × ")
                    })
                    .filter(|value| !value.is_empty())
                    .unwrap_or_else(|| "—".into());
                let hole = match aperture["hasHole"].as_bool() {
                    Some(true) => "да",
                    Some(false) => "нет",
                    None => "зависит от макроса",
                };
                writeln!(
                    stdout,
                    "  D{}: {} ({modifiers}); отверстие: {hole}; использований: {}",
                    aperture["dCode"], aperture["template"], aperture["usageCount"]
                )?;
            }
            Ok(())
        }
        Action::ImageMeta { .. } => {
            writeln!(stdout, "Формат: {}", value["format"])?;
            writeln!(stdout, "MIME: {}", value["mimeType"])?;
            writeln!(stdout, "Размер файла: {} байт", value["fileSize"])?;
            writeln!(
                stdout,
                "Разрешение: {} × {} px ({:.2} Мп)",
                value["width"],
                value["height"],
                value["megapixels"].as_f64().unwrap_or(0.0)
            )?;
            writeln!(stdout, "Соотношение сторон: {}", value["aspectRatio"])?;
            writeln!(
                stdout,
                "Цвет: {}; {} бит/пиксель; {} каналов; альфа: {}",
                value["colorType"].as_str().unwrap_or("неизвестно"),
                value["bitsPerPixel"],
                value["channelCount"],
                if value["hasAlpha"].as_bool().unwrap_or(false) {
                    "да"
                } else {
                    "нет"
                }
            )?;
            let labels = |key: &str| {
                match key {
                    "cameraMake" => "Производитель камеры",
                    "cameraModel" => "Модель камеры",
                    "lensModel" => "Объектив",
                    "dateTaken" => "Дата съёмки",
                    "dateDigitized" => "Дата оцифровки",
                    "software" => "Программа",
                    "artist" => "Автор",
                    "copyright" => "Авторские права",
                    "exposureTime" => "Выдержка",
                    "aperture" => "Диафрагма",
                    "iso" => "ISO",
                    "focalLength" => "Фокусное расстояние",
                    "focalLength35mm" => "Эквивалент 35 мм",
                    "exposureBias" => "Экспокоррекция",
                    "flash" => "Вспышка",
                    "orientation" => "Ориентация",
                    "whiteBalance" => "Баланс белого",
                    "gpsPosition" => "GPS",
                    _ => key,
                }
                .to_owned()
            };
            if value["exif"].as_array().is_some_and(Vec::is_empty) {
                writeln!(stdout, "EXIF: не найден")?;
            } else {
                writeln!(stdout, "EXIF:")?;
                for field in value["exif"].as_array().into_iter().flatten() {
                    writeln!(
                        stdout,
                        "  {}: {}",
                        labels(field["key"].as_str().unwrap_or("")),
                        field["value"].as_str().unwrap_or("")
                    )?;
                }
            }
            Ok(())
        }
    }
}

fn report_error(error: &ConvertError, json_output: bool) -> ExitCode {
    let (code, name) = match error {
        ConvertError::Cancelled => (130, "cancelled"),
        ConvertError::InvalidInput(_) => (2, "invalid_input"),
        ConvertError::UnsupportedFormat(_) => (2, "unsupported_format"),
        ConvertError::IncompatibleCodec(_) => (2, "incompatible_codec"),
        ConvertError::Io(_) => (1, "io_error"),
        ConvertError::ProcessFailed(_) => (1, "process_failed"),
    };
    if json_output {
        let _ = print_json(&json!({"ok":false,"error":{"code":name,"message":error.to_string()}}));
    } else {
        let _ = writeln!(io::stderr(), "{error}");
    }
    ExitCode::from(code)
}

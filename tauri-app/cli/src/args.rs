use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;
use uniconvert_core::audio::AudioOptions;

#[derive(Debug, Parser)]
#[command(
    name = "uniconvert",
    version,
    about = "Локальная конвертация медиа и извлечение метаданных"
)]
pub struct Cli {
    /// Один JSON-объект в stdout; без прогресса. Ошибки также выводятся в JSON.
    #[arg(long, global = true)]
    pub json: bool,
    /// Не показывать прогресс в stderr.
    #[arg(long, global = true)]
    pub quiet: bool,
    #[command(subcommand)]
    pub command: Action,
}

#[derive(Debug, Subcommand)]
pub enum Action {
    /// Конвертировать аудио. Старое имя команды: convert.
    #[command(visible_alias = "convert")]
    Audio(AudioArgs),
    /// Конвертировать изображение.
    Image(ImageArgs),
    /// Конвертировать видео через FFmpeg.
    Video(VideoArgs),
    /// Показать информацию об аудиофайле и его первой аудиодорожке.
    Probe { input: PathBuf },
    /// Извлечь основные атрибуты и параметры Gerber X2/X3.
    GerberMeta { input: PathBuf },
    /// Извлечь технические свойства и полезные EXIF-метаданные изображения.
    ImageMeta { input: PathBuf },
}

#[derive(Debug, Args)]
pub struct AudioArgs {
    pub input: PathBuf,
    /// Формат результата: mp3, wav, flac, aac, ogg, opus, m4a, wma, aiff.
    #[arg(long = "to", value_name = "FORMAT")]
    pub format: String,
    /// Папка результата (по умолчанию рядом с исходником).
    #[arg(short = 'o', long, value_name = "DIR")]
    pub output_dir: Option<PathBuf>,
    /// Битрейт, кбит/с. По умолчанию выбирает кодировщик; неприменим к WAV/FLAC/AIFF.
    #[arg(long, value_name = "KBPS")]
    pub bitrate: Option<u32>,
    /// Частота дискретизации, Гц. По умолчанию выбирает кодировщик.
    #[arg(long, value_name = "HZ")]
    pub sample_rate: Option<u32>,
    /// 1 — моно, 2 — стерео; по умолчанию исходные каналы.
    #[arg(long)]
    pub channels: Option<u8>,
    /// Изменение громкости, от -60 до 30 дБ.
    #[arg(long, allow_hyphen_values = true, value_name = "DB")]
    pub volume: Option<f32>,
    /// Темп от 0.5 до 2.0; высота тона сохраняется.
    #[arg(long, value_name = "MULTIPLIER")]
    pub tempo: Option<f32>,
    /// Сдвиг высоты тона от -12 до 12 полутонов; длительность сохраняется.
    #[arg(long, allow_hyphen_values = true, value_name = "SEMITONES")]
    pub pitch: Option<f32>,
    /// Включить LUFS-нормализацию (по умолчанию -14 LUFS).
    #[arg(long)]
    pub normalize: bool,
    /// Целевой LUFS при нормализации, от -70 до -5.
    #[arg(long, requires = "normalize", allow_hyphen_values = true)]
    pub target_lufs: Option<f32>,
    #[arg(long)]
    pub strip_metadata: bool,
    /// Начало: секунды, мм:сс.ммм или чч:мм:сс.ммм.
    #[arg(long, value_parser = parse_timestamp, value_name = "TIME")]
    pub start: Option<f64>,
    /// Конец относительно исходной записи (не длительность фрагмента).
    #[arg(long, value_parser = parse_timestamp, value_name = "TIME")]
    pub end: Option<f64>,
}

impl AudioArgs {
    pub fn options(&self) -> AudioOptions {
        AudioOptions {
            format: self.format.clone(),
            bitrate_kbps: self.bitrate,
            sample_rate_hz: self.sample_rate,
            channels: self.channels,
            volume_db: self.volume,
            tempo: self.tempo,
            pitch_semitones: self.pitch,
            normalize_lufs: self.normalize,
            target_lufs: self.target_lufs,
            strip_metadata: self.strip_metadata,
            trim_start: self.start,
            trim_end: self.end,
        }
    }
}

#[derive(Debug, Args)]
pub struct ImageArgs {
    pub input: PathBuf,
    /// Формат результата: jpg, png, webp, avif, gif, bmp, tiff, ico.
    #[arg(long = "to", value_name = "FORMAT")]
    pub format: String,
    /// Папка результата (по умолчанию рядом с исходником).
    #[arg(short = 'o', long, value_name = "DIR")]
    pub output_dir: Option<PathBuf>,
    /// Качество кодирования от 0 до 100.
    #[arg(long, default_value_t = 80, value_parser = clap::value_parser!(u8).range(0..=100))]
    pub quality: u8,
    /// Не переносить EXIF и ICC-профиль.
    #[arg(long)]
    pub strip_metadata: bool,
}

#[derive(Debug, Args)]
pub struct VideoArgs {
    pub input: PathBuf,
    /// Контейнер результата: mp4, mkv, webm, mov, avi.
    #[arg(long = "to", value_name = "FORMAT")]
    pub format: String,
    /// Папка результата (по умолчанию рядом с исходником).
    #[arg(short = 'o', long, value_name = "DIR")]
    pub output_dir: Option<PathBuf>,
    /// Видеокодек: h264, h265, vp9, av1.
    #[arg(long, default_value = "h264")]
    pub video_codec: String,
    /// Качество видео от 0 до 51 (меньше — выше качество).
    #[arg(long, default_value_t = 23, value_parser = clap::value_parser!(u8).range(0..=51))]
    pub crf: u8,
    /// Высота результата: 480, 720, 1080, 1440 или 2160.
    #[arg(long, value_parser = parse_video_height)]
    pub height: Option<u32>,
    /// Частота кадров: 24, 30, 50 или 60.
    #[arg(long, value_parser = parse_video_fps)]
    pub fps: Option<u32>,
    /// Аудиокодек: aac, mp3, opus, flac.
    #[arg(long, default_value = "aac")]
    pub audio_codec: String,
    /// Битрейт аудио, кбит/с.
    #[arg(long, default_value_t = 192, value_parser = clap::value_parser!(u32).range(8..=1536))]
    pub audio_bitrate: u32,
    #[arg(long)]
    pub strip_metadata: bool,
    /// Копировать потоки без перекодирования; остальные настройки кодеков игнорируются.
    #[arg(long)]
    pub copy_streams: bool,
}

fn parse_timestamp(text: &str) -> Result<f64, String> {
    let invalid =
        || "Ожидается неотрицательное время: секунды, мм:сс.ммм или чч:мм:сс.ммм".to_owned();
    let normalized = text.trim().replace(',', ".");
    let parts: Vec<_> = normalized.split(':').collect();
    if parts.is_empty() || parts.len() > 3 {
        return Err(invalid());
    }
    let mut seconds = 0.0;
    for (index, part) in parts.iter().enumerate() {
        let last = index + 1 == parts.len();
        let pieces: Vec<_> = part.split('.').collect();
        if pieces.len() > if last { 2 } else { 1 }
            || pieces
                .iter()
                .any(|piece| piece.is_empty() || !piece.bytes().all(|byte| byte.is_ascii_digit()))
            || (pieces.len() == 2 && pieces[1].len() > 3)
        {
            return Err(invalid());
        }
        let value: f64 = part.parse().map_err(|_| invalid())?;
        if !value.is_finite() || (index > 0 && value >= 60.0) {
            return Err(invalid());
        }
        seconds = seconds * 60.0 + value;
    }
    if !seconds.is_finite() {
        return Err(invalid());
    }
    Ok(seconds)
}

fn parse_video_height(text: &str) -> Result<u32, String> {
    let value = text
        .parse::<u32>()
        .map_err(|_| "Высота должна быть числом".to_owned())?;
    [480, 720, 1080, 1440, 2160]
        .contains(&value)
        .then_some(value)
        .ok_or_else(|| "Допустимая высота: 480, 720, 1080, 1440 или 2160".to_owned())
}

fn parse_video_fps(text: &str) -> Result<u32, String> {
    let value = text
        .parse::<u32>()
        .map_err(|_| "FPS должен быть числом".to_owned())?;
    [24, 30, 50, 60]
        .contains(&value)
        .then_some(value)
        .ok_or_else(|| "Допустимый FPS: 24, 30, 50 или 60".to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_precise_timestamps_and_negative_audio_levels() {
        assert_eq!(parse_timestamp("1:02:03,125").unwrap(), 3723.125);
        assert_eq!(parse_timestamp("62:03.125").unwrap(), 3723.125);
        for text in [
            "", "-1", "NaN", "1:60", "1.5:00", "1:2:3:4", "1.0001", "1e3",
        ] {
            assert!(parse_timestamp(text).is_err(), "{text}");
        }
        let cli = Cli::try_parse_from([
            "uniconvert",
            "convert",
            "a.wav",
            "--to",
            "mp3",
            "--volume",
            "-3",
            "--normalize",
            "--target-lufs",
            "-16",
            "--json",
        ])
        .unwrap();
        assert!(cli.json);
        let Action::Audio(args) = cli.command else {
            panic!()
        };
        assert_eq!(args.options().volume_db, Some(-3.0));
        assert!(args.options().convert_options().is_ok());
        assert!(Cli::try_parse_from([
            "uniconvert",
            "convert",
            "a.wav",
            "--to",
            "mp3",
            "--target-lufs",
            "-16"
        ])
        .is_err());
    }
}

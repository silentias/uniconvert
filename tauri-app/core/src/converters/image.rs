use super::Converter;
use std::{fs::File, io::BufWriter, time::Instant};

use crate::domain::{ConvertError, ConvertOutput, ConvertRequest, MediaKind};
use image::{
    codecs::{
        avif::AvifEncoder, jpeg::JpegEncoder, png::PngEncoder, tiff::TiffEncoder, webp::WebPEncoder,
    },
    DynamicImage, ImageDecoder, ImageEncoder, ImageFormat, ImageReader,
};

#[derive(Default)]
struct ImageMetadata {
    exif: Option<Vec<u8>>,
    icc: Option<Vec<u8>>,
}

pub struct ImageConverter;

impl Converter for ImageConverter {
    fn supports(&self, kind: MediaKind, request: &ConvertRequest) -> bool {
        // TODO: Сначала принимать распространённые изображения.
        // TODO: Затем учитывать анимацию, HDR и цветовые профили.
        kind == MediaKind::Image && image_format(request.options.container).is_some()
    }

    fn convert(&self, request: &ConvertRequest) -> Result<ConvertOutput, ConvertError> {
        // TODO: Сначала перекодировать изображение без преобразований.
        // TODO: Затем добавить resize, метаданные и оптимизаторы.
        let started_at = Instant::now();
        let (image, metadata) = load_image(&request.input, request.options.strip_metadata)?;
        let format = image_format(request.options.container).ok_or_else(|| {
            ConvertError::UnsupportedFormat(request.options.container.extension().into())
        })?;

        let quality = request.options.quality.unwrap_or(80).min(100);
        encode_image(&image, &request.output, format, quality, metadata)?;

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

fn load_image(
    path: &std::path::Path,
    strip_metadata: bool,
) -> Result<(DynamicImage, ImageMetadata), ConvertError> {
    // TODO: Сейчас переносить EXIF/ICC; затем добавить XMP, IPTC и выбор отдельных групп.
    let reader = ImageReader::open(path)
        .map_err(|error| ConvertError::InvalidInput(error.to_string()))?
        .with_guessed_format()
        .map_err(|error| ConvertError::InvalidInput(error.to_string()))?;
    let mut decoder = reader
        .into_decoder()
        .map_err(|error| ConvertError::InvalidInput(error.to_string()))?;
    let orientation = decoder
        .orientation()
        .map_err(|error| ConvertError::InvalidInput(error.to_string()))?;
    let metadata = if strip_metadata {
        ImageMetadata::default()
    } else {
        ImageMetadata {
            exif: decoder.exif_metadata().unwrap_or(None),
            icc: decoder.icc_profile().unwrap_or(None),
        }
    };
    let mut image = DynamicImage::from_decoder(decoder)
        .map_err(|error| ConvertError::InvalidInput(error.to_string()))?;

    if strip_metadata {
        image.apply_orientation(orientation);
    }

    Ok((image, metadata))
}

fn encode_image(
    image: &DynamicImage,
    output: &std::path::Path,
    format: ImageFormat,
    quality: u8,
    metadata: ImageMetadata,
) -> Result<(), ConvertError> {
    // TODO: Сейчас сохранять поддерживаемые EXIF/ICC; затем сообщать о частично потерянных типах metadata.
    let writer = || {
        File::create(output)
            .map(BufWriter::new)
            .map_err(|error| ConvertError::Io(error.to_string()))
    };
    let encoding_result = match format {
        ImageFormat::Jpeg => {
            let mut encoder = JpegEncoder::new_with_quality(writer()?, quality.max(1));
            apply_metadata(&mut encoder, &metadata, true)?;
            image.write_with_encoder(encoder)
        }
        ImageFormat::Avif => {
            let mut encoder = AvifEncoder::new_with_speed_quality(writer()?, 6, quality);
            if let Some(exif) = metadata.exif {
                encoder
                    .set_exif_metadata(exif)
                    .map_err(|error| ConvertError::ProcessFailed(error.to_string()))?;
            }
            image.write_with_encoder(encoder)
        }
        ImageFormat::Png => {
            let mut encoder = PngEncoder::new(writer()?);
            apply_metadata(&mut encoder, &metadata, true)?;
            image.write_with_encoder(encoder)
        }
        ImageFormat::WebP => {
            let mut encoder = WebPEncoder::new_lossless(writer()?);
            apply_metadata(&mut encoder, &metadata, true)?;
            image.write_with_encoder(encoder)
        }
        ImageFormat::Tiff => {
            let mut encoder = TiffEncoder::new(writer()?);
            apply_metadata(&mut encoder, &metadata, false)?;
            image.write_with_encoder(encoder)
        }
        _ => image.save_with_format(output, format),
    };

    encoding_result.map_err(|error| ConvertError::ProcessFailed(error.to_string()))
}

fn apply_metadata(
    encoder: &mut impl ImageEncoder,
    metadata: &ImageMetadata,
    include_exif: bool,
) -> Result<(), ConvertError> {
    if include_exif {
        if let Some(exif) = &metadata.exif {
            encoder
                .set_exif_metadata(exif.clone())
                .map_err(|error| ConvertError::ProcessFailed(error.to_string()))?;
        }
    }
    if let Some(icc) = &metadata.icc {
        encoder
            .set_icc_profile(icc.clone())
            .map_err(|error| ConvertError::ProcessFailed(error.to_string()))?;
    }
    Ok(())
}

fn image_format(container: crate::domain::Container) -> Option<ImageFormat> {
    // TODO: Позже учитывать возможности конкретной сборки и параметры энкодера.
    match container {
        crate::domain::Container::Jpeg => Some(ImageFormat::Jpeg),
        crate::domain::Container::Png => Some(ImageFormat::Png),
        crate::domain::Container::Webp => Some(ImageFormat::WebP),
        crate::domain::Container::Avif => Some(ImageFormat::Avif),
        crate::domain::Container::Gif => Some(ImageFormat::Gif),
        crate::domain::Container::Bmp => Some(ImageFormat::Bmp),
        crate::domain::Container::Tiff => Some(ImageFormat::Tiff),
        crate::domain::Container::Ico => Some(ImageFormat::Ico),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use image::{Rgba, RgbaImage};

    use super::*;
    use crate::domain::{CompressionMode, Container, ConvertOptions};

    #[test]
    fn converts_image_to_every_exposed_format() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let input = directory.path().join("source.png");
        RgbaImage::from_pixel(2, 2, Rgba([40, 120, 220, 255]))
            .save(&input)
            .expect("source image");

        for container in [
            Container::Jpeg,
            Container::Png,
            Container::Webp,
            Container::Avif,
            Container::Gif,
            Container::Bmp,
            Container::Tiff,
            Container::Ico,
        ] {
            let request = ConvertRequest {
                input: input.clone(),
                output: directory
                    .path()
                    .join(format!("result.{}", container.extension())),
                options: ConvertOptions {
                    container,
                    video_codec: None,
                    audio_codec: None,
                    compression: CompressionMode::Custom,
                    quality: Some(80),
                    strip_metadata: true,
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
                },
            };

            let output = ImageConverter.convert(&request).expect("converted image");
            assert!(output.path.is_file());
            assert!(output.output_size > 0);
        }
    }
}

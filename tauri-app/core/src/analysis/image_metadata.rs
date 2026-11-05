use crate::domain::ConvertError;
use exif::{Exif, In, Reader, Tag, Value};
use image::ImageReader;
use serde::{Deserialize, Serialize};
use std::{fs, io::BufReader, path::Path};

const MAX_IMAGE_SIZE: u64 = 128 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageMetadata {
    pub format: String,
    pub mime_type: String,
    pub file_size: u64,
    pub width: u32,
    pub height: u32,
    pub megapixels: f64,
    pub aspect_ratio: String,
    pub color_type: String,
    pub bits_per_pixel: u16,
    pub channel_count: u8,
    pub has_alpha: bool,
    pub exif: Vec<ImageMetadataField>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageMetadataField {
    pub key: String,
    pub value: String,
}

pub fn extract_metadata(path: &Path) -> Result<ImageMetadata, ConvertError> {
    let file_metadata = fs::metadata(path)
        .map_err(|error| ConvertError::InvalidInput(format!("{}: {error}", path.display())))?;
    if !file_metadata.is_file() {
        return Err(ConvertError::InvalidInput(
            "Ожидается локальный файл изображения".into(),
        ));
    }
    if file_metadata.len() > MAX_IMAGE_SIZE {
        return Err(ConvertError::InvalidInput(
            "Изображение превышает допустимый размер 128 МБ".into(),
        ));
    }

    let reader = ImageReader::open(path)
        .map_err(|error| ConvertError::InvalidInput(error.to_string()))?
        .with_guessed_format()
        .map_err(|error| ConvertError::InvalidInput(error.to_string()))?;
    let format = reader
        .format()
        .ok_or_else(|| ConvertError::InvalidInput("Не удалось определить формат изображения".into()))?;
    let image = reader
        .decode()
        .map_err(|error| ConvertError::InvalidInput(format!("Не удалось прочитать изображение: {error}")))?;
    let color = image.color();
    let (width, height) = (image.width(), image.height());

    Ok(ImageMetadata {
        format: format_name(format).into(),
        mime_type: mime_type(format).into(),
        file_size: file_metadata.len(),
        width,
        height,
        megapixels: width as f64 * height as f64 / 1_000_000.0,
        aspect_ratio: aspect_ratio(width, height),
        color_type: color_type_name(color).into(),
        bits_per_pixel: color.bits_per_pixel(),
        channel_count: color.channel_count(),
        has_alpha: color.has_alpha(),
        exif: read_exif(path),
    })
}

fn read_exif(path: &Path) -> Vec<ImageMetadataField> {
    let Ok(file) = fs::File::open(path) else {
        return Vec::new();
    };
    let Ok(exif) = Reader::new().read_from_container(&mut BufReader::new(file)) else {
        return Vec::new();
    };

    let mut fields = Vec::new();
    for (key, tag) in [
        ("cameraMake", Tag::Make),
        ("cameraModel", Tag::Model),
        ("lensModel", Tag::LensModel),
        ("dateTaken", Tag::DateTimeOriginal),
        ("dateDigitized", Tag::DateTimeDigitized),
        ("software", Tag::Software),
        ("artist", Tag::Artist),
        ("copyright", Tag::Copyright),
        ("exposureTime", Tag::ExposureTime),
        ("aperture", Tag::FNumber),
        ("iso", Tag::PhotographicSensitivity),
        ("focalLength", Tag::FocalLength),
        ("focalLength35mm", Tag::FocalLengthIn35mmFilm),
        ("exposureBias", Tag::ExposureBiasValue),
        ("flash", Tag::Flash),
        ("orientation", Tag::Orientation),
        ("whiteBalance", Tag::WhiteBalance),
    ] {
        if let Some(value) = field_text(&exif, tag) {
            fields.push(ImageMetadataField {
                key: key.into(),
                value,
            });
        }
    }
    if let Some(value) = gps_position(&exif) {
        fields.push(ImageMetadataField {
            key: "gpsPosition".into(),
            value,
        });
    }
    fields
}

fn field_text(exif: &Exif, tag: Tag) -> Option<String> {
    let field = exif.get_field(tag, In::PRIMARY)?;
    let value = match &field.value {
        Value::Ascii(values) => values
            .first()
            .map(|value| String::from_utf8_lossy(value).trim_matches(['\0', ' ']).to_owned()),
        _ => Some(field.display_value().with_unit(exif).to_string()),
    }?;
    (!value.trim().is_empty()).then_some(value)
}

fn gps_position(exif: &Exif) -> Option<String> {
    let latitude = gps_coordinate(exif.get_field(Tag::GPSLatitude, In::PRIMARY)?)?;
    let longitude = gps_coordinate(exif.get_field(Tag::GPSLongitude, In::PRIMARY)?)?;
    let latitude = if field_text(exif, Tag::GPSLatitudeRef)?.to_ascii_uppercase() == "S" {
        -latitude
    } else {
        latitude
    };
    let longitude = if field_text(exif, Tag::GPSLongitudeRef)?.to_ascii_uppercase() == "W" {
        -longitude
    } else {
        longitude
    };
    Some(format!("{latitude:.6}, {longitude:.6}"))
}

fn gps_coordinate(field: &exif::Field) -> Option<f64> {
    let Value::Rational(values) = &field.value else {
        return None;
    };
    let value = |index: usize| {
        let rational = values.get(index)?;
        (rational.denom != 0).then_some(rational.num as f64 / rational.denom as f64)
    };
    Some(value(0)? + value(1)? / 60.0 + value(2)? / 3600.0)
}

fn format_name(format: image::ImageFormat) -> &'static str {
    match format {
        image::ImageFormat::Jpeg => "JPEG",
        image::ImageFormat::Png => "PNG",
        image::ImageFormat::Gif => "GIF",
        image::ImageFormat::WebP => "WebP",
        image::ImageFormat::Tiff => "TIFF",
        image::ImageFormat::Bmp => "BMP",
        image::ImageFormat::Ico => "ICO",
        image::ImageFormat::Avif => "AVIF",
        _ => "Другое",
    }
}

fn mime_type(format: image::ImageFormat) -> &'static str {
    match format {
        image::ImageFormat::Jpeg => "image/jpeg",
        image::ImageFormat::Png => "image/png",
        image::ImageFormat::Gif => "image/gif",
        image::ImageFormat::WebP => "image/webp",
        image::ImageFormat::Tiff => "image/tiff",
        image::ImageFormat::Bmp => "image/bmp",
        image::ImageFormat::Ico => "image/x-icon",
        image::ImageFormat::Avif => "image/avif",
        _ => "application/octet-stream",
    }
}

fn color_type_name(color: image::ColorType) -> &'static str {
    match color {
        image::ColorType::L8 | image::ColorType::L16 => "Оттенки серого",
        image::ColorType::La8 | image::ColorType::La16 => "Оттенки серого + альфа",
        image::ColorType::Rgb8 | image::ColorType::Rgb16 | image::ColorType::Rgb32F => "RGB",
        image::ColorType::Rgba8 | image::ColorType::Rgba16 | image::ColorType::Rgba32F => "RGBA",
        _ => "Другая",
    }
}

fn aspect_ratio(width: u32, height: u32) -> String {
    fn gcd(mut left: u32, mut right: u32) -> u32 {
        while right != 0 {
            (left, right) = (right, left % right);
        }
        left.max(1)
    }
    let divisor = gcd(width, height);
    format!("{}:{}", width / divisor, height / divisor)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Rgba, RgbaImage};

    #[test]
    fn reads_useful_technical_metadata_without_exif() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("transparent.png");
        RgbaImage::from_pixel(1920, 1080, Rgba([1, 2, 3, 128]))
            .save(&path)
            .unwrap();

        let metadata = extract_metadata(&path).unwrap();
        assert_eq!(metadata.format, "PNG");
        assert_eq!(metadata.mime_type, "image/png");
        assert_eq!((metadata.width, metadata.height), (1920, 1080));
        assert_eq!(metadata.aspect_ratio, "16:9");
        assert_eq!(metadata.color_type, "RGBA");
        assert_eq!(metadata.bits_per_pixel, 32);
        assert_eq!(metadata.channel_count, 4);
        assert!(metadata.has_alpha);
        assert!(metadata.exif.is_empty());
    }

    #[test]
    fn reads_camera_fields_from_jpeg_exif() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("camera.jpg");
        image::RgbImage::from_pixel(2, 1, image::Rgb([10, 20, 30]))
            .save(&path)
            .unwrap();

        let original = std::fs::read(&path).unwrap();
        let mut tiff = Vec::new();
        tiff.extend_from_slice(b"II\x2a\0\x08\0\0\0");
        tiff.extend_from_slice(&2_u16.to_le_bytes());
        for (tag, offset) in [(0x010f_u16, 38_u32), (0x0110_u16, 44_u32)] {
            tiff.extend_from_slice(&tag.to_le_bytes());
            tiff.extend_from_slice(&2_u16.to_le_bytes());
            tiff.extend_from_slice(&6_u32.to_le_bytes());
            tiff.extend_from_slice(&offset.to_le_bytes());
        }
        tiff.extend_from_slice(&0_u32.to_le_bytes());
        tiff.extend_from_slice(b"Canon\0EOS R\0");
        let mut app1 = b"Exif\0\0".to_vec();
        app1.extend_from_slice(&tiff);
        let segment_length = (app1.len() + 2) as u16;
        let mut jpeg = original[..2].to_vec();
        jpeg.extend_from_slice(&[0xff, 0xe1]);
        jpeg.extend_from_slice(&segment_length.to_be_bytes());
        jpeg.extend_from_slice(&app1);
        jpeg.extend_from_slice(&original[2..]);
        std::fs::write(&path, jpeg).unwrap();

        let metadata = extract_metadata(&path).unwrap();
        assert!(metadata
            .exif
            .iter()
            .any(|field| field.key == "cameraMake" && field.value == "Canon"));
        assert!(metadata
            .exif
            .iter()
            .any(|field| field.key == "cameraModel" && field.value == "EOS R"));
    }
}

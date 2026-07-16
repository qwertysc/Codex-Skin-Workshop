use image::ImageReader;
use image::imageops::FilterType;
use serde::Serialize;
use std::{collections::HashMap, fs, path::{Path, PathBuf}};
use uuid::Uuid;

use crate::{theme::ThemeStore, AppError};

const MAX_IMAGE_BYTES: u64 = 25 * 1024 * 1024;
const MAX_PIXELS: u64 = 40_000_000;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportedImage {
    pub relative_path: String,
    pub absolute_path: PathBuf,
    pub width: u32,
    pub height: u32,
    pub palette: Vec<String>,
}

pub fn import_image(store: &ThemeStore, source: &Path) -> Result<ImportedImage, AppError> {
    let meta = fs::metadata(source)?;
    if !meta.is_file() || meta.len() > MAX_IMAGE_BYTES {
        return Err(AppError::Validation("图片必须是普通文件，且不能超过 25 MiB".into()));
    }
    let reader = ImageReader::open(source)?.with_guessed_format()?;
    let format = reader.format().ok_or_else(|| AppError::Validation("无法识别图片格式".into()))?;
    let extension = match format {
        image::ImageFormat::Png => "png",
        image::ImageFormat::Jpeg => "jpg",
        image::ImageFormat::WebP => "webp",
        image::ImageFormat::Gif => "gif",
        _ => return Err(AppError::Validation("仅支持 PNG、JPEG、WebP 和 GIF 图片".into())),
    };
    let (width, height) = reader.into_dimensions()?;
    if u64::from(width) * u64::from(height) > MAX_PIXELS {
        return Err(AppError::Validation("图片尺寸过大".into()));
    }
    let image = ImageReader::open(source)?.with_guessed_format()?.decode()?;
    // Re-encode the first decoded frame. This strips metadata and makes imported bytes trusted image data.
    let name = format!("{}.{}", Uuid::new_v4(), extension);
    let relative_path = format!("images/{name}");
    let destination = store.root().join(&relative_path);
    image.save_with_format(&destination, format)?;
    Ok(ImportedImage {
        relative_path,
        absolute_path: destination,
        width,
        height,
        palette: extract_palette(&image, 6),
    })
}

fn extract_palette(image: &image::DynamicImage, count: usize) -> Vec<String> {
    let thumb = image.resize(96, 96, FilterType::Triangle).to_rgb8();
    let mut buckets: HashMap<(u8, u8, u8), u32> = HashMap::new();
    for pixel in thumb.pixels() {
        let key = (pixel[0] >> 4, pixel[1] >> 4, pixel[2] >> 4);
        *buckets.entry(key).or_default() += 1;
    }
    let mut colors: Vec<_> = buckets.into_iter().collect();
    colors.sort_by_key(|(_, n)| std::cmp::Reverse(*n));
    colors.into_iter().take(count).map(|((r, g, b), _)| {
        format!("#{:02x}{:02x}{:02x}", r * 17, g * 17, b * 17)
    }).collect()
}

pub fn resolve_imported_image(root: &Path, relative: &str) -> Result<PathBuf, AppError> {
    if !relative.starts_with("images/") || relative.contains("..") || relative.contains('\\') || relative.contains('\0') {
        return Err(AppError::Validation("导入图片路径无效".into()));
    }
    Ok(root.join(relative))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn palette_prefers_dominant_color() {
        let mut img = image::RgbImage::new(10, 10);
        for p in img.pixels_mut() { *p = image::Rgb([250, 5, 5]); }
        let palette = extract_palette(&image::DynamicImage::ImageRgb8(img), 3);
        assert_eq!(palette[0], "#ff0000");
    }
    #[test]
    fn imported_path_cannot_escape() {
        assert!(resolve_imported_image(Path::new("/safe"), "images/../../x").is_err());
    }
}

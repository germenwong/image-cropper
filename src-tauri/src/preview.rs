use crate::crop;
use crate::model::{CropConfig, PreviewInfo};
use base64::{engine::general_purpose::STANDARD, Engine};
use image::imageops;
use std::path::Path;

/// 预览最长边限制，避免超大图在 webview 中卡顿
const PREVIEW_MAX: u32 = 1200;

fn to_data_url(bytes: &[u8], mime: &str) -> String {
    format!("data:{};base64,{}", mime, STANDARD.encode(bytes))
}

fn downscale(img: image::RgbaImage) -> image::RgbaImage {
    let (w, h) = (img.width(), img.height());
    if w <= PREVIEW_MAX && h <= PREVIEW_MAX {
        return img;
    }
    let scale = PREVIEW_MAX as f64 / w.max(h) as f64;
    let nw = ((w as f64 * scale).round() as u32).max(1);
    let nh = ((h as f64 * scale).round() as u32).max(1);
    imageops::resize(&img, nw, nh, imageops::FilterType::Triangle)
}

pub fn origin_preview(abs_path: &str) -> Result<PreviewInfo, String> {
    let img = image::open(abs_path).map_err(|e| format!("无法读取图片: {}", e))?;
    let (width, height) = (img.width(), img.height());
    let small = downscale(img.to_rgba8());
    let mut buf = std::io::Cursor::new(Vec::new());
    image::DynamicImage::ImageRgba8(small)
        .write_to(&mut buf, image::ImageFormat::Png)
        .map_err(|e| e.to_string())?;
    let rel = Path::new(abs_path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    Ok(PreviewInfo {
        abs_path: abs_path.to_string(),
        rel_path: rel,
        original: to_data_url(buf.get_ref(), "image/png"),
        width,
        height,
    })
}

/// 渲染裁切结果预览（限制输出尺寸，保持目标宽高比）
pub fn render(abs_path: &str, cfg: &CropConfig) -> Result<String, String> {
    let img = image::open(abs_path).map_err(|e| format!("无法读取图片: {}", e))?;
    let out = crop::crop_to(&img, cfg.target_w, cfg.target_h, &cfg.fill);
    let scale = (PREVIEW_MAX as f64 / out.width().max(out.height()) as f64).min(1.0);
    let final_img = if scale < 1.0 {
        let nw = ((out.width() as f64 * scale).round() as u32).max(1);
        let nh = ((out.height() as f64 * scale).round() as u32).max(1);
        imageops::resize(&out, nw, nh, imageops::FilterType::Triangle)
    } else {
        out
    };
    let mut buf = std::io::Cursor::new(Vec::new());
    image::DynamicImage::ImageRgba8(final_img)
        .write_to(&mut buf, image::ImageFormat::Png)
        .map_err(|e| e.to_string())?;
    Ok(to_data_url(buf.get_ref(), "image/png"))
}

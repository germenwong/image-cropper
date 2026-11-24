use crate::model::{CropConfig, FileEntry, FillMode, ScanResult};
use image::{imageops, DynamicImage, ImageBuffer, Rgba, RgbaImage};
use std::path::Path;
use walkdir::WalkDir;

const SUPPORTED: [&str; 4] = ["png", "jpg", "jpeg", "webp"];

pub fn is_supported(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| SUPPORTED.contains(&e.to_ascii_lowercase().as_str()))
        .unwrap_or(false)
}

pub fn scan_dir(root: &str) -> ScanResult {
    let mut files = Vec::new();
    for entry in WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if !entry.file_type().is_file() {
            continue;
        }
        let abs = entry.path();
        if !is_supported(abs) {
            continue;
        }
        // 跳过隐藏文件（如 .DS_Store 附属文件）
        if abs
            .file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.starts_with('.'))
            .unwrap_or(false)
        {
            continue;
        }
        let rel = abs
            .strip_prefix(root)
            .unwrap_or(abs)
            .to_string_lossy()
            .to_string();
        files.push(FileEntry {
            rel_path: rel,
            abs_path: abs.to_string_lossy().to_string(),
        });
    }
    files.sort_by(|a, b| a.rel_path.cmp(&b.rel_path));
    ScanResult {
        total: files.len(),
        files,
    }
}

pub fn parse_color(hex: &str) -> Rgba<u8> {
    let h = hex.trim().trim_start_matches('#');
    let parse = |s: &str, d: u8| u8::from_str_radix(s, 16).unwrap_or(d);
    match h.len() {
        6 => Rgba([
            parse(&h[0..2], 255),
            parse(&h[2..4], 255),
            parse(&h[4..6], 255),
            255,
        ]),
        // 8 位 rgba
        8 => Rgba([
            parse(&h[0..2], 255),
            parse(&h[2..4], 255),
            parse(&h[4..6], 255),
            parse(&h[6..8], 255),
        ]),
        // 3 位 rgb
        3 => Rgba([
            parse(&h[0..1], 0) * 17,
            parse(&h[1..2], 0) * 17,
            parse(&h[2..3], 0) * 17,
            255,
        ]),
        _ => Rgba([255, 255, 255, 255]),
    }
}

fn fill_color(fill: &FillMode) -> Rgba<u8> {
    match fill {
        FillMode::Transparent => Rgba([0, 0, 0, 0]),
        FillMode::Color { hex } => parse_color(hex),
        FillMode::Stretch => Rgba([0, 0, 0, 0]),
    }
}

/// 计算裁切时原图在目标画布中的可见区域（用于前端叠加裁切框）
/// 返回 (x, y, w, h)，坐标基于等比缩放后铺满目标画布时的原图坐标系
pub fn crop_rect(ow: u32, oh: u32, tw: u32, th: u32) -> (f64, f64, f64, f64) {
    if ow == 0 || oh == 0 || tw == 0 || th == 0 {
        return (0.0, 0.0, ow as f64, oh as f64);
    }
    // 原图先 cover 缩放到目标画布（超出部分被裁掉）
    let scale = f64::max(tw as f64 / ow as f64, th as f64 / oh as f64);
    let vis_w = (tw as f64 / scale).min(ow as f64);
    let vis_h = (th as f64 / scale).min(oh as f64);
    // 原图小于目标时 cover 比例 < 1，等价于居中放置，可见区域即整图
    ((ow as f64 - vis_w) / 2.0, (oh as f64 - vis_h) / 2.0, vis_w, vis_h)
}

/// 核心：将原图按配置裁切/填充到目标尺寸，返回 RGBA 图像
pub fn crop_to(img: &DynamicImage, tw: u32, th: u32, fill: &FillMode) -> RgbaImage {
    let src = img.to_rgba8();
    let (ow, oh) = (src.width(), src.height());
    if tw == 0 || th == 0 || ow == 0 || oh == 0 {
        return src;
    }

    // 拉伸模式：直接铺满
    if matches!(fill, FillMode::Stretch) {
        return imageops::resize(&src, tw, th, imageops::FilterType::Lanczos3);
    }

    let mut canvas: RgbaImage = ImageBuffer::from_pixel(tw, th, fill_color(fill));

    // 原图整体小于目标尺寸：不放大，居中放置
    let shrink = ow > tw || oh > th;
    let (nw, nh) = if shrink {
        let scale = f64::max(tw as f64 / ow as f64, th as f64 / oh as f64);
        (
            ((ow as f64 * scale).round() as u32).max(1),
            ((oh as f64 * scale).round() as u32).max(1),
        )
    } else {
        (ow, oh)
    };

    let scaled = if shrink {
        imageops::resize(&src, nw, nh, imageops::FilterType::Lanczos3)
    } else {
        src.clone()
    };

    // 居中放置并按目标尺寸裁剪多余区域
    let sx = (scaled.width().saturating_sub(tw)) / 2;
    let sy = (scaled.height().saturating_sub(th)) / 2;
    let dx = (tw.saturating_sub(scaled.width())) / 2;
    let dy = (th.saturating_sub(scaled.height())) / 2;
    let cw = tw.min(scaled.width());
    let ch = th.min(scaled.height());

    let cropped = imageops::crop_imm(&scaled, sx, sy, cw, ch).to_image();
    // 已填充背景的画布不做覆盖混合，直接把裁剪结果贴到居中位置
    imageops::replace(&mut canvas, &cropped, dx as i64, dy as i64);
    canvas
}

/// 导出编码
pub fn encode(img: &RgbaImage, cfg: &CropConfig) -> image::ImageResult<Vec<u8>> {
    use crate::model::OutputFormat;
    use std::io::Cursor;
    let mut buf = Cursor::new(Vec::new());
    match cfg.format {
        OutputFormat::Png => {
            DynamicImage::ImageRgba8(img.clone())
                .write_to(&mut buf, image::ImageFormat::Png)?;
        }
        OutputFormat::Jpg => {
            // jpg 不支持 alpha，透明区域铺白
            let out = flatten_on(img, Rgba([255, 255, 255, 255]));
            let mut enc = image::codecs::jpeg::JpegEncoder::new_with_quality(
                &mut buf,
                cfg.quality.clamp(1, 100),
            );
            enc.encode_image(&DynamicImage::ImageRgb8(out))?;
        }
        OutputFormat::Webp => {
            // 透明区域铺白后输出无损 webp（image 0.25 仅支持无损 webp 编码）
            let flat = flatten_on(img, Rgba([255, 255, 255, 255]));
            let enc = image::codecs::webp::WebPEncoder::new_lossless(&mut buf);
            enc.encode(
                flat.as_raw(),
                flat.width(),
                flat.height(),
                image::ExtendedColorType::Rgb8,
            )?;
        }
    }
    Ok(buf.into_inner())
}

fn flatten_on(img: &RgbaImage, bg: Rgba<u8>) -> image::RgbImage {
    let mut out = image::RgbImage::new(img.width(), img.height());
    for (x, y, p) in img.enumerate_pixels() {
        let a = p[3] as u32;
        // 简单 alpha over
        let r = (p[0] as u32 * a + bg[0] as u32 * (255 - a)) / 255;
        let g = (p[1] as u32 * a + bg[1] as u32 * (255 - a)) / 255;
        let b = (p[2] as u32 * a + bg[2] as u32 * (255 - a)) / 255;
        *out.get_pixel_mut(x, y) = image::Rgb([r as u8, g as u8, b as u8]);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn solid(w: u32, h: u32, c: [u8; 4]) -> DynamicImage {
        DynamicImage::ImageRgba8(ImageBuffer::from_pixel(w, h, Rgba(c)))
    }

    #[test]
    fn wide_image_crops_sides() {
        let img = solid(400, 100, [255, 0, 0, 255]);
        let out = crop_to(&img, 100, 100, &FillMode::Transparent);
        assert_eq!((out.width(), out.height()), (100, 100));
        assert_eq!(out.get_pixel(50, 50)[0], 255);
        assert_eq!(out.get_pixel(50, 50)[3], 255);
    }

    #[test]
    fn tall_image_crops_top_bottom() {
        let img = solid(100, 400, [0, 255, 0, 255]);
        let out = crop_to(&img, 100, 100, &FillMode::Transparent);
        assert_eq!((out.width(), out.height()), (100, 100));
        assert_eq!(out.get_pixel(50, 50)[1], 255);
    }

    #[test]
    fn small_image_is_padded_not_stretched() {
        let img = solid(50, 50, [0, 0, 255, 255]);
        let out = crop_to(&img, 100, 100, &FillMode::Color { hex: "#FFFFFF".into() });
        assert_eq!((out.width(), out.height()), (100, 100));
        // 边缘是白色背景
        let p = out.get_pixel(2, 2);
        assert_eq!([p[0], p[1], p[2]], [255, 255, 255]);
        // 中心是原图
        assert_eq!(out.get_pixel(50, 50)[2], 255);
    }

    #[test]
    fn stretch_fills_whole_canvas() {
        let img = solid(400, 100, [255, 0, 0, 255]);
        let out = crop_to(&img, 100, 100, &FillMode::Stretch);
        assert_eq!((out.width(), out.height()), (100, 100));
        assert_eq!(out.get_pixel(0, 99)[3], 255);
    }

    #[test]
    fn color_parse() {
        assert_eq!(parse_color("#ff0000"), Rgba([255, 0, 0, 255]));
        assert_eq!(parse_color("00ff00"), Rgba([0, 255, 0, 255]));
        assert_eq!(parse_color("#fff"), Rgba([255, 255, 255, 255]));
    }
}

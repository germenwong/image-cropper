use crate::model::{CropConfig, CropMode, FileEntry, FillMode, ScanResult};
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
pub fn crop_rect(
    ow: u32,
    oh: u32,
    tw: u32,
    th: u32,
    mode: CropMode,
    allow_upscale: bool,
) -> (f64, f64, f64, f64) {
    if ow == 0 || oh == 0 || tw == 0 || th == 0 {
        return (0.0, 0.0, ow as f64, oh as f64);
    }
    let (vis_w, vis_h) = match mode {
        // 像素裁切：按目标像素 1:1 居中截取，原图不足时取整图
        CropMode::Pixel => ((tw as f64).min(ow as f64), (th as f64).min(oh as f64)),
        CropMode::Spec => {
            // 小图且不允许放大：整图保留
            if !allow_upscale && ow <= tw && oh <= th {
                (ow as f64, oh as f64)
            } else {
                // cover 缩放后居中裁切，超出部分被裁掉
                let scale = f64::max(tw as f64 / ow as f64, th as f64 / oh as f64);
                (
                    (tw as f64 / scale).min(ow as f64),
                    (th as f64 / scale).min(oh as f64),
                )
            }
        }
    };
    // 与 crop_to 的整数取整保持一致，避免框与导出结果差半像素
    (
        ((ow as f64 - vis_w) / 2.0).floor(),
        ((oh as f64 - vis_h) / 2.0).floor(),
        vis_w,
        vis_h,
    )
}

/// 规格输出触发放大时的提示文案（导出日志用）
pub fn upscale_note(img_w: u32, img_h: u32, cfg: &CropConfig) -> Option<String> {
    if cfg.mode != CropMode::Spec
        || !cfg.allow_upscale
        || matches!(cfg.fill, FillMode::Stretch)
        || img_w == 0
        || img_h == 0
    {
        return None;
    }
    let scale = f64::max(
        cfg.target_w as f64 / img_w as f64,
        cfg.target_h as f64 / img_h as f64,
    );
    if scale > 1.02 {
        Some(format!("已放大 {:.1}×，画质会下降", scale))
    } else {
        None
    }
}

/// 核心：将原图按配置裁切/填充到目标尺寸，返回 RGBA 图像
///
/// - 规格输出（Spec）：先按目标比例 cover 取景，再缩放到目标像素；小图默认不放大
/// - 像素裁切（Pixel）：按目标像素 1:1 从原图居中截取，不缩放
/// - 拉伸铺满：忽略比例直接铺满，优先于以上两种模式
pub fn crop_to(img: &DynamicImage, cfg: &CropConfig) -> RgbaImage {
    let (tw, th) = (cfg.target_w, cfg.target_h);
    let src = img.to_rgba8();
    let (ow, oh) = (src.width(), src.height());
    if tw == 0 || th == 0 || ow == 0 || oh == 0 {
        return src;
    }

    // 拉伸模式：直接铺满
    if matches!(cfg.fill, FillMode::Stretch) {
        return imageops::resize(&src, tw, th, imageops::FilterType::Lanczos3);
    }

    let mut canvas: RgbaImage = ImageBuffer::from_pixel(tw, th, fill_color(&cfg.fill));

    // 是否需要 cover 缩放：像素裁切始终不缩放；规格输出在大图或允许放大时缩放
    let cover = match cfg.mode {
        CropMode::Pixel => false,
        CropMode::Spec => cfg.allow_upscale || ow > tw || oh > th,
    };

    let (scaled, nw, nh) = if cover {
        let scale = f64::max(tw as f64 / ow as f64, th as f64 / oh as f64);
        let nw = ((ow as f64 * scale).round() as u32).max(1);
        let nh = ((oh as f64 * scale).round() as u32).max(1);
        (imageops::resize(&src, nw, nh, imageops::FilterType::Lanczos3), nw, nh)
    } else {
        (src.clone(), ow, oh)
    };

    // 居中取景（多余部分裁掉，不足部分留填充）
    let sx = nw.saturating_sub(tw) / 2;
    let sy = nh.saturating_sub(th) / 2;
    let cw = tw.min(nw);
    let ch = th.min(nh);
    let dx = tw.saturating_sub(cw) / 2;
    let dy = th.saturating_sub(ch) / 2;

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

    fn cfg(tw: u32, th: u32, fill: FillMode) -> CropConfig {
        CropConfig {
            src_dir: String::new(),
            out_dir: String::new(),
            target_w: tw,
            target_h: th,
            mode: CropMode::Spec,
            allow_upscale: false,
            fill,
            format: crate::model::OutputFormat::Png,
            quality: 90,
        }
    }

    #[test]
    fn wide_image_crops_sides() {
        let img = solid(400, 100, [255, 0, 0, 255]);
        let out = crop_to(&img, &cfg(100, 100, FillMode::Transparent));
        assert_eq!((out.width(), out.height()), (100, 100));
        assert_eq!(out.get_pixel(50, 50)[0], 255);
        assert_eq!(out.get_pixel(50, 50)[3], 255);
    }

    #[test]
    fn tall_image_crops_top_bottom() {
        let img = solid(100, 400, [0, 255, 0, 255]);
        let out = crop_to(&img, &cfg(100, 100, FillMode::Transparent));
        assert_eq!((out.width(), out.height()), (100, 100));
        assert_eq!(out.get_pixel(50, 50)[1], 255);
    }

    #[test]
    fn small_image_is_padded_not_stretched() {
        let img = solid(50, 50, [0, 0, 255, 255]);
        let out = crop_to(&img, &cfg(100, 100, FillMode::Color { hex: "#FFFFFF".into() }));
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
        let out = crop_to(&img, &cfg(100, 100, FillMode::Stretch));
        assert_eq!((out.width(), out.height()), (100, 100));
        assert_eq!(out.get_pixel(0, 99)[3], 255);
    }

    #[test]
    fn spec_mode_upscales_only_when_allowed() {
        let img = solid(50, 50, [0, 0, 255, 255]);

        // 默认不放大：中心是原图，四角是填充
        let out = crop_to(&img, &cfg(100, 100, FillMode::Transparent));
        assert_eq!(out.get_pixel(2, 2)[3], 0);
        assert_eq!(out.get_pixel(50, 50)[3], 255);

        // 允许放大：铺满，无透明像素
        let mut c = cfg(100, 100, FillMode::Transparent);
        c.allow_upscale = true;
        let out = crop_to(&img, &c);
        assert_eq!(out.get_pixel(0, 0)[3], 255);
        assert_eq!(out.get_pixel(99, 99)[3], 255);
    }

    #[test]
    fn pixel_mode_crops_target_pixels_without_scaling() {
        // 1600x2133 -> 100x100：从中心 1:1 取 100x100 像素
        let img = solid(1600, 2133, [10, 20, 30, 255]);
        let mut c = cfg(100, 100, FillMode::Transparent);
        c.mode = CropMode::Pixel;
        let out = crop_to(&img, &c);
        assert_eq!((out.width(), out.height()), (100, 100));
        assert_eq!(out.get_pixel(50, 50)[3], 255);

        // 原图比目标小：整图保留，其余填充
        let small = solid(40, 40, [10, 20, 30, 255]);
        let out = crop_to(&small, &c);
        assert_eq!(out.get_pixel(1, 1)[3], 0);
        assert_eq!(out.get_pixel(50, 50)[3], 255);
    }

    #[test]
    fn pixel_mode_keeps_original_detail() {
        // 中心 100x100 的原图区域被原样保留：左半红、右半绿
        let mut img = ImageBuffer::from_pixel(400, 400, Rgba([255, 0, 0, 255]));
        for (x, _y, p) in img.enumerate_pixels_mut() {
            if x >= 200 {
                *p = Rgba([0, 255, 0, 255]);
            }
        }
        let img = DynamicImage::ImageRgba8(img);
        let mut c = cfg(100, 100, FillMode::Transparent);
        c.mode = CropMode::Pixel;
        let out = crop_to(&img, &c);
        // 中心 100px 落在 x=150..250，一半红一半绿
        assert_eq!(out.get_pixel(10, 50)[0], 255);
        assert_eq!(out.get_pixel(90, 50)[1], 255);
    }

    #[test]
    fn crop_rect_matches_small_image_padding() {
        // 小图不放大：整图保留，框应为全图
        assert_eq!(
            crop_rect(100, 50, 100, 100, CropMode::Spec, false),
            (0.0, 0.0, 100.0, 50.0)
        );
        assert_eq!(
            crop_rect(50, 50, 100, 100, CropMode::Spec, false),
            (0.0, 0.0, 50.0, 50.0)
        );
    }

    #[test]
    fn crop_rect_keeps_full_width_for_wide_source() {
        // 1600x2133 -> 100x100：保留全宽 + 中间 1600 高度
        let (x, y, w, h) = crop_rect(1600, 2133, 100, 100, CropMode::Spec, false);
        assert_eq!((x, w), (0.0, 1600.0));
        assert_eq!(h, 1600.0);
        assert_eq!(y, 266.0);
    }

    #[test]
    fn crop_rect_pixel_mode_is_target_size() {
        // 像素裁切：框就是目标像素大小，居中
        let (x, y, w, h) = crop_rect(1600, 2133, 100, 100, CropMode::Pixel, false);
        assert_eq!((w, h), (100.0, 100.0));
        assert_eq!((x, y), (750.0, 1016.0));
    }

    #[test]
    fn upscale_note_only_when_allowed() {
        let mut c = cfg(100, 100, FillMode::Transparent);
        // 未开启放大：无提示
        assert_eq!(upscale_note(50, 50, &c), None);
        // 开启放大且确实放大：有提示
        c.allow_upscale = true;
        assert!(upscale_note(50, 50, &c).is_some());
        // 大图不需要放大：无提示
        assert_eq!(upscale_note(200, 200, &c), None);
        // 像素裁切模式：永不放大
        let mut pixel = cfg(100, 100, FillMode::Transparent);
        pixel.mode = CropMode::Pixel;
        pixel.allow_upscale = true;
        assert_eq!(upscale_note(50, 50, &pixel), None);
    }

    #[test]
    fn color_parse() {
        assert_eq!(parse_color("#ff0000"), Rgba([255, 0, 0, 255]));
        assert_eq!(parse_color("00ff00"), Rgba([0, 255, 0, 255]));
        assert_eq!(parse_color("#fff"), Rgba([255, 255, 255, 255]));
    }
}

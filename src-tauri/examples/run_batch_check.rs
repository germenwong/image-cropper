// 端到端批处理验证：读取 testdata/src，导出到 testdata/out，校验结果
use image::imageops;
use std::path::{Path, PathBuf};

fn unique_path(dir: &Path, stem: &str, ext: &str) -> PathBuf {
    let mut p = dir.join(format!("{}.{}", stem, ext));
    let mut i = 1;
    while p.exists() {
        p = dir.join(format!("{}_{}.{}", stem, i, ext));
        i += 1;
    }
    p
}

/// 按需求文档规则：cover 缩放 + 居中裁切，小图居中填充
fn crop_center(img: &image::DynamicImage, tw: u32, th: u32, bg: image::Rgba<u8>) -> image::RgbaImage {
    let src = img.to_rgba8();
    let (ow, oh) = (src.width(), src.height());
    let mut canvas: image::RgbaImage = image::ImageBuffer::from_pixel(tw, th, bg);

    let (nw, nh) = if ow > tw || oh > th {
        let s = f64::max(tw as f64 / ow as f64, th as f64 / oh as f64);
        (((ow as f64 * s).round() as u32).max(1), ((oh as f64 * s).round() as u32).max(1))
    } else {
        (ow, oh)
    };
    let scaled = if (nw, nh) == (ow, oh) {
        src.clone()
    } else {
        imageops::resize(&src, nw, nh, imageops::FilterType::Lanczos3)
    };
    let sx = scaled.width().saturating_sub(tw) / 2;
    let sy = scaled.height().saturating_sub(th) / 2;
    let cw = tw.min(scaled.width());
    let ch = th.min(scaled.height());
    let cropped = imageops::crop_imm(&scaled, sx, sy, cw, ch).to_image();
    let dx = (tw.saturating_sub(cw)) / 2;
    let dy = (th.saturating_sub(ch)) / 2;
    imageops::replace(&mut canvas, &cropped, dx as i64, dy as i64);
    canvas
}

fn main() {
    let src_root = Path::new("testdata/src");
    let out_root = Path::new("testdata/out");
    let _ = std::fs::remove_dir_all(out_root);
    std::fs::create_dir_all(out_root).unwrap();

    let (tw, th) = (512u32, 512u32);
    let mut ok = 0;
    let mut failed: Vec<(String, String)> = Vec::new();

    let mut files: Vec<PathBuf> = walkdir::WalkDir::new(src_root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| e.path().to_path_buf())
        .filter(|p| {
            matches!(
                p.extension().and_then(|s| s.to_str()).map(|s| s.to_ascii_lowercase()).as_deref(),
                Some("png" | "jpg" | "jpeg" | "webp")
            )
        })
        .collect();
    files.sort();
    println!("扫描到 {} 张图片", files.len());

    for f in &files {
        let rel = f.strip_prefix(src_root).unwrap();
        let res = (|| -> Result<(), String> {
            let img = image::open(f).map_err(|e| format!("读取失败: {}", e))?;
            let out = crop_center(&img, tw, th, image::Rgba([0, 0, 0, 0]));
            let parent = out_root.join(rel.parent().unwrap());
            std::fs::create_dir_all(&parent).map_err(|e| e.to_string())?;
            let stem = rel.file_stem().unwrap().to_string_lossy().to_string();
            let target = unique_path(&parent, &stem, "png");
            out.save(&target).map_err(|e| format!("保存失败: {}", e))?;
            Ok(())
        })();
        match res {
            Ok(_) => ok += 1,
            Err(e) => failed.push((rel.to_string_lossy().to_string(), e)),
        }
    }

    println!("成功 {} 张，失败 {} 张", ok, failed.len());
    for (n, r) in &failed {
        println!("  失败: {} -> {}", n, r);
    }

    // 校验输出尺寸与目录层级
    let mut out_files = Vec::new();
    for e in walkdir::WalkDir::new(out_root).into_iter().filter_map(|e| e.ok()) {
        if e.file_type().is_file() {
            out_files.push(e.path().to_path_buf());
        }
    }
    out_files.sort();
    println!("\n输出文件 {} 个:", out_files.len());
    let mut bad = 0;
    for p in &out_files {
        let img = image::open(p).unwrap();
        let ok_size = img.width() == tw && img.height() == th;
        if !ok_size {
            bad += 1;
        }
        println!(
            "  {}{}",
            p.strip_prefix(out_root).unwrap().display(),
            if ok_size { "" } else { "  <尺寸异常!>" }
        );
    }
    println!("\n尺寸异常 {} 个", bad);

    // 透明素材应保留 alpha
    let transparent = out_root.join("子目录A/透明素材.png");
    if transparent.exists() {
        let img = image::open(&transparent).unwrap().to_rgba8();
        let corner = img.get_pixel(2, 2);
        println!("透明素材角落像素 alpha = {}（应为 0）", corner[3]);
    }
}

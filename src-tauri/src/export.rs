use crate::crop;
use crate::model::*;
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter};

pub const EV_PROGRESS: &str = "export://progress";
pub const EV_LOG: &str = "export://log";
pub const EV_DONE: &str = "export://done";

/// 重名探测：name.ext -> name_1.ext -> name_2.ext
pub fn unique_path(dir: &Path, stem: &str, ext: &str) -> PathBuf {
    let mut p = dir.join(format!("{}.{}", stem, ext));
    let mut i = 1;
    while p.exists() {
        p = dir.join(format!("{}_{}.{}", stem, i, ext));
        i += 1;
    }
    p
}

pub fn process_one(entry: &FileEntry, cfg: &CropConfig) -> Result<PathBuf, String> {
    let img = image::open(&entry.abs_path).map_err(|e| format!("无法读取图片: {}", e))?;
    let out_img = crop::crop_to(&img, cfg.target_w, cfg.target_h, &cfg.fill);
    let bytes = crop::encode(&out_img, cfg).map_err(|e| format!("编码失败: {}", e))?;

    let rel = Path::new(&entry.rel_path);
    let stem = rel
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "image".into());
    let parent_rel = rel.parent().unwrap_or(Path::new(""));
    let out_parent = Path::new(&cfg.out_dir).join(parent_rel);
    std::fs::create_dir_all(&out_parent).map_err(|e| format!("无法创建目录: {}", e))?;

    let target = {
        let _guard = WRITE_LOCK.lock().unwrap();
        let target = unique_path(&out_parent, &stem, cfg.format.ext());
        std::fs::write(&target, bytes).map_err(|e| format!("写入失败: {}", e))?;
        target
    };
    Ok(target)
}

/// 同一目录下并发写文件时，需要串行化重名探测 + 写入
static WRITE_LOCK: Mutex<()> = Mutex::new(());

pub fn run_export(
    app: AppHandle,
    cfg: CropConfig,
    files: Vec<FileEntry>,
    cancel: Arc<AtomicBool>,
    task_id: u64,
) {
    let total = files.len();
    let done = Arc::new(AtomicUsize::new(0));
    let failed = Arc::new(AtomicUsize::new(0));
    let cancelled = AtomicBool::new(false);

    let _ = app.emit(
        EV_PROGRESS,
        ProgressPayload {
            task_id,
            total,
            done: 0,
            failed: 0,
            current: String::new(),
        },
    );

    // 并发度：限制在 CPU 核数内，避免同时解码过多大图
    let threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
        .min(8);
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build()
        .expect("线程池创建失败");

    pool.install(|| {
        files.par_iter().for_each(|entry| {
            if cancel.load(Ordering::Relaxed) {
                cancelled.store(true, Ordering::Relaxed);
                return;
            }
            let result = process_one(entry, &cfg);
            match result {
                Ok(_) => {
                    let _ = app.emit(
                        EV_LOG,
                        LogPayload {
                            task_id,
                            ok: true,
                            name: entry.rel_path.clone(),
                            reason: None,
                        },
                    );
                }
                Err(reason) => {
                    failed.fetch_add(1, Ordering::Relaxed);
                    let _ = app.emit(
                        EV_LOG,
                        LogPayload {
                            task_id,
                            ok: false,
                            name: entry.rel_path.clone(),
                            reason: Some(reason),
                        },
                    );
                }
            }
            let d = done.fetch_add(1, Ordering::Relaxed) + 1;
            let _ = app.emit(
                EV_PROGRESS,
                ProgressPayload {
                    task_id,
                    total,
                    done: d,
                    failed: failed.load(Ordering::Relaxed),
                    current: entry.rel_path.clone(),
                },
            );
        });
    });

    let _ = app.emit(
        EV_DONE,
        DonePayload {
            task_id,
            total,
            done: done.load(Ordering::Relaxed),
            failed: failed.load(Ordering::Relaxed),
            cancelled: cancelled.load(Ordering::Relaxed) || cancel.load(Ordering::Relaxed),
            out_dir: cfg.out_dir.clone(),
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{FillMode, OutputFormat};
    use image::{ImageBuffer, Rgb, RgbImage, Rgba, RgbaImage};

    fn temp_root(name: &str) -> std::path::PathBuf {
        let p = std::env::temp_dir().join(format!("imgcrop-test-{}-{}", name, std::process::id()));
        let _ = std::fs::remove_dir_all(&p);
        std::fs::create_dir_all(&p).unwrap();
        p
    }

    fn make_images(root: &Path) {
        std::fs::create_dir_all(root.join("中文目录/深层")).unwrap();
        let rgb: RgbImage = ImageBuffer::from_pixel(800, 400, Rgb([200, 30, 30]));
        rgb.save(root.join("宽图.jpg")).unwrap();
        rgb.save(root.join("中文目录/宽图.jpg")).unwrap(); // 跨目录同名
        let rgba: RgbaImage = ImageBuffer::from_pixel(300, 300, Rgba([0, 200, 0, 255]));
        rgba.save(root.join("中文目录/深层/透明.png")).unwrap();
        std::fs::write(root.join("坏图.jpg"), b"broken").unwrap();
    }

    #[test]
    fn export_pipeline_end_to_end() {
        let base = temp_root("e2e");
        let src = base.join("素材");
        let out = base.join("输出");
        make_images(&src);
        std::fs::create_dir_all(&out).unwrap();

        let files = crate::crop::scan_dir(src.to_str().unwrap()).files;
        assert_eq!(files.len(), 4, "应扫描到 4 张图片");

        let cfg = CropConfig {
            src_dir: src.to_string_lossy().to_string(),
            out_dir: out.to_string_lossy().to_string(),
            target_w: 256,
            target_h: 256,
            fill: FillMode::Color { hex: "#ffffff".into() },
            format: OutputFormat::Png,
            quality: 90,
        };

        // 不依赖 AppHandle：直接复用 process_one
        let results: Vec<_> = files.iter().map(|f| process_one(f, &cfg)).collect();
        let failures = results.iter().filter(|r| r.is_err()).count();
        assert_eq!(failures, 1, "坏图应失败");

        // 目录层级保留
        assert!(out.join("中文目录/深层/透明.png").exists());
        // 跨目录同名文件各自保留
        assert!(out.join("宽图.png").exists());
        assert!(out.join("中文目录/宽图.png").exists());

        // 输出尺寸正确
        let img = image::open(out.join("宽图.png")).unwrap();
        assert_eq!((img.width(), img.height()), (256, 256));

        // 小图填充：300x300 -> 256x256 不放大，只裁切
        assert!(out.join("中文目录/深层/透明.png").exists());

        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn duplicate_names_get_suffix() {
        let base = temp_root("dup");
        std::fs::create_dir_all(&base).unwrap();
        std::fs::write(base.join("a.png"), b"x").unwrap();
        let p = unique_path(&base, "a", "png");
        assert_eq!(p.file_name().unwrap(), "a_1.png");
        let _ = std::fs::remove_dir_all(&base);
    }
}

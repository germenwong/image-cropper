mod crop;
mod export;
mod model;
mod presets;
mod preview;

use model::*;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, State};

struct AppState {
    cancel: Arc<AtomicBool>,
    running: Arc<Mutex<bool>>,
    task_seq: AtomicU64,
}

#[tauri::command]
async fn scan_images(dir: String) -> Result<ScanResult, String> {
    tauri::async_runtime::spawn_blocking(move || Ok(crop::scan_dir(&dir)))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn pick_random_preview(dir: String) -> Result<Option<PreviewInfo>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let files = crop::scan_dir(&dir).files;
        if files.is_empty() {
            return Ok(None);
        }
        use rand::seq::SliceRandom;
        let mut rng = rand::thread_rng();
        let pick = files.choose(&mut rng).unwrap();
        let info = preview::origin_preview(&pick.abs_path)?;
        Ok(Some(info))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn file_preview(path: String) -> Result<PreviewInfo, String> {
    tauri::async_runtime::spawn_blocking(move || preview::origin_preview(&path))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn render_preview(path: String, cfg: CropConfig) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || preview::render(&path, &cfg))
        .await
        .map_err(|e| e.to_string())?
}

fn validate(cfg: &CropConfig) -> Result<(), String> {
    if cfg.src_dir.trim().is_empty() || cfg.out_dir.trim().is_empty() {
        return Err("请先选择素材目录和输出目录".into());
    }
    if cfg.target_w == 0 || cfg.target_h == 0 {
        return Err("目标宽高必须大于 0".into());
    }
    // 输出目录可能尚未创建，先建出来再比较
    std::fs::create_dir_all(&cfg.out_dir).map_err(|e| format!("无法创建输出目录: {}", e))?;
    let src = std::fs::canonicalize(&cfg.src_dir).map_err(|_| "素材目录不存在")?;
    let out = std::fs::canonicalize(&cfg.out_dir).map_err(|_| "输出目录不存在")?;
    if src == out {
        return Err("素材目录与输出目录不能相同".into());
    }
    // 输出目录位于素材目录内部会造成重复扫描，同样禁止
    if out.starts_with(&src) || src.starts_with(&out) {
        return Err("素材目录与输出目录不能相互包含".into());
    }
    Ok(())
}

#[tauri::command]
async fn start_export(
    app: AppHandle,
    state: State<'_, AppState>,
    cfg: CropConfig,
) -> Result<u64, String> {
    validate(&cfg)?;
    {
        let mut running = state.running.lock().unwrap();
        if *running {
            return Err("已有导出任务进行中".into());
        }
        *running = true;
    }
    state.cancel.store(false, Ordering::SeqCst);
    let task_id = state.task_seq.fetch_add(1, Ordering::SeqCst) + 1;
    let cancel = state.cancel.clone();
    let running = state.running.clone();

    let scan = crop::scan_dir(&cfg.src_dir);
    if scan.files.is_empty() {
        *running.lock().unwrap() = false;
        return Err("素材目录中没有可处理的图片".into());
    }

    let app2 = app.clone();
    std::thread::spawn(move || {
        export::run_export(app2, cfg, scan.files, cancel, task_id);
        *running.lock().unwrap() = false;
    });
    Ok(task_id)
}

#[tauri::command]
fn cancel_export(state: State<'_, AppState>) {
    state.cancel.store(true, Ordering::SeqCst);
}

#[tauri::command]
fn reveal_in_dir(app: AppHandle, path: String) -> Result<(), String> {
    use tauri_plugin_opener::OpenerExt;
    app.opener()
        .open_path(path, None::<&str>)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn load_presets(app: AppHandle) -> Vec<Preset> {
    presets::load(&app)
}

#[tauri::command]
fn save_presets(app: AppHandle, presets: Vec<Preset>) -> Result<(), String> {
    presets::save(&app, &presets)
}

#[tauri::command]
fn crop_rect(
    ow: u32,
    oh: u32,
    tw: u32,
    th: u32,
    mode: CropMode,
    allow_upscale: bool,
) -> (f64, f64, f64, f64) {
    crop::crop_rect(ow, oh, tw, th, mode, allow_upscale)
}

/// 统计素材目录信息（供前端展示）
#[tauri::command]
async fn count_images(dir: String) -> Result<usize, String> {
    tauri::async_runtime::spawn_blocking(move || Ok(crop::scan_dir(&dir).total))
        .await
        .map_err(|e| e.to_string())?
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .manage(AppState {
            cancel: Arc::new(AtomicBool::new(false)),
            running: Arc::new(Mutex::new(false)),
            task_seq: AtomicU64::new(0),
        })
        .invoke_handler(tauri::generate_handler![
            scan_images,
            pick_random_preview,
            file_preview,
            render_preview,
            start_export,
            cancel_export,
            reveal_in_dir,
            load_presets,
            save_presets,
            count_images,
            crop_rect
        ])
        .setup(|_app| {
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

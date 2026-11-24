use crate::model::Preset;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

fn preset_file(app: &AppHandle) -> Option<PathBuf> {
    let dir = app.path().app_config_dir().ok()?;
    std::fs::create_dir_all(&dir).ok()?;
    Some(dir.join("presets.json"))
}

pub fn load(app: &AppHandle) -> Vec<Preset> {
    let Some(p) = preset_file(app) else {
        return Vec::new();
    };
    std::fs::read_to_string(p)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save(app: &AppHandle, presets: &[Preset]) -> Result<(), String> {
    let p = preset_file(app).ok_or("无法获取配置目录")?;
    let s = serde_json::to_string_pretty(presets).map_err(|e| e.to_string())?;
    std::fs::write(p, s).map_err(|e| e.to_string())
}

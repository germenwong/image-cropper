use serde::{Deserialize, Serialize};

/// 填充策略
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum FillMode {
    /// 透明填充
    Transparent,
    /// 白/黑/自定义色值（#RRGGBB 或 #RRGGBBAA）
    Color { hex: String },
    /// 拉伸铺满（忽略宽高比）
    Stretch,
}

impl Default for FillMode {
    fn default() -> Self {
        FillMode::Transparent
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    Png,
    Jpg,
    Webp,
}

impl OutputFormat {
    pub fn ext(&self) -> &'static str {
        match self {
            OutputFormat::Png => "png",
            OutputFormat::Jpg => "jpg",
            OutputFormat::Webp => "webp",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CropConfig {
    pub src_dir: String,
    pub out_dir: String,
    pub target_w: u32,
    pub target_h: u32,
    pub fill: FillMode,
    pub format: OutputFormat,
    /// 0~100，仅 jpg / webp 生效
    pub quality: u8,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileEntry {
    /// 相对素材根目录的路径
    pub rel_path: String,
    pub abs_path: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanResult {
    pub files: Vec<FileEntry>,
    pub total: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewInfo {
    pub abs_path: String,
    pub rel_path: String,
    /// 原图 data url
    pub original: String,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgressPayload {
    pub task_id: u64,
    pub total: usize,
    pub done: usize,
    pub failed: usize,
    pub current: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogPayload {
    pub task_id: u64,
    pub ok: bool,
    pub name: String,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DonePayload {
    pub task_id: u64,
    pub total: usize,
    pub done: usize,
    pub failed: usize,
    pub cancelled: bool,
    pub out_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Preset {
    pub id: String,
    pub name: String,
    pub w: u32,
    pub h: u32,
}

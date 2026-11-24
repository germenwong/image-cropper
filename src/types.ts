export type OutputFormat = "png" | "jpg" | "webp";

export type FillMode =
  | { kind: "transparent" }
  | { kind: "color"; hex: string }
  | { kind: "stretch" };

export interface CropConfig {
  srcDir: string;
  outDir: string;
  targetW: number;
  targetH: number;
  fill: FillMode;
  format: OutputFormat;
  quality: number;
}

export interface FileEntry {
  relPath: string;
  absPath: string;
}

export interface ScanResult {
  files: FileEntry[];
  total: number;
}

export interface PreviewInfo {
  absPath: string;
  relPath: string;
  original: string;
  width: number;
  height: number;
}

export interface ProgressPayload {
  taskId: number;
  total: number;
  done: number;
  failed: number;
  current: string;
}

export interface LogPayload {
  taskId: number;
  ok: boolean;
  name: string;
  reason: string | null;
}

export interface DonePayload {
  taskId: number;
  total: number;
  done: number;
  failed: number;
  cancelled: boolean;
  outDir: string;
}

export interface Preset {
  id: string;
  name: string;
  w: number;
  h: number;
}

import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  CropConfig,
  DonePayload,
  LogPayload,
  Preset,
  PreviewInfo,
  ProgressPayload,
  ScanResult,
} from "./types";

export const api = {
  scanImages: (dir: string) => invoke<ScanResult>("scan_images", { dir }),
  countImages: (dir: string) => invoke<number>("count_images", { dir }),
  pickRandomPreview: (dir: string) =>
    invoke<PreviewInfo | null>("pick_random_preview", { dir }),
  filePreview: (path: string) => invoke<PreviewInfo>("file_preview", { path }),
  renderPreview: (path: string, cfg: CropConfig) =>
    invoke<string>("render_preview", { path, cfg }),
  startExport: (cfg: CropConfig) => invoke<number>("start_export", { cfg }),
  cancelExport: () => invoke<void>("cancel_export"),
  revealInDir: (path: string) => invoke<void>("reveal_in_dir", { path }),
  loadPresets: () => invoke<Preset[]>("load_presets"),
  savePresets: (presets: Preset[]) => invoke<void>("save_presets", { presets }),
  cropRect: (ow: number, oh: number, tw: number, th: number) =>
    invoke<[number, number, number, number]>("crop_rect", { ow, oh, tw, th }),
};

export const events = {
  onProgress: (cb: (p: ProgressPayload) => void): Promise<UnlistenFn> =>
    listen<ProgressPayload>("export://progress", (e) => cb(e.payload)),
  onLog: (cb: (p: LogPayload) => void): Promise<UnlistenFn> =>
    listen<LogPayload>("export://log", (e) => cb(e.payload)),
  onDone: (cb: (p: DonePayload) => void): Promise<UnlistenFn> =>
    listen<DonePayload>("export://done", (e) => cb(e.payload)),
};

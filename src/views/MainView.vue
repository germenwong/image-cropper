<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, reactive, ref, watch } from "vue";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import {
  NButton,
  NCard,
  NDivider,
  NEmpty,
  NIcon,
  NProgress,
  NScrollbar,
  NSwitch,
  NTag,
  useDialog,
  useMessage,
} from "naive-ui";
import {
  CheckmarkCircleOutline,
  CloseCircleOutline,
  FolderOpenOutline,
  PlayOutline,
  StopOutline,
} from "@vicons/ionicons5";
import DirPicker from "../components/DirPicker.vue";
import ParamPanel from "../components/ParamPanel.vue";
import PreviewPane from "../components/PreviewPane.vue";
import { api, events } from "../api";
import { isDark } from "../theme";
import type {
  CropConfig,
  DonePayload,
  LogPayload,
  Preset,
  PreviewInfo,
  ProgressPayload,
} from "../types";

const message = useMessage();
const dialog = useDialog();

const config = reactive<CropConfig>({
  srcDir: "",
  outDir: "",
  targetW: 512,
  targetH: 512,
  mode: "spec",
  allowUpscale: false,
  fill: { kind: "transparent" },
  format: "png",
  quality: 90,
});

const presets = ref<Preset[]>([]);
const srcCount = ref<number | undefined>(undefined);
const dragTarget = ref<"" | "src" | "out" | "preview">("");

// 预览状态
const samplePath = ref("");
const originalPreview = ref<PreviewInfo | null>(null);
const croppedPreview = ref("");
const cropBox = ref<{ x: number; y: number; w: number; h: number } | null>(null);
const previewing = ref(false);
const dirty = ref(false);
const autoPreview = ref(true);

// 导出状态
const running = ref(false);
const progress = reactive<ProgressPayload>({
  taskId: 0,
  total: 0,
  done: 0,
  failed: 0,
  current: "",
});
const logs = ref<LogPayload[]>([]);
const failCount = computed(() => progress.failed);
const percent = computed(() =>
  progress.total ? Math.floor((progress.done / progress.total) * 100) : 0
);
const onlyFailed = ref(false);
const visibleLogs = computed(() => {
  const list = onlyFailed.value ? logs.value.filter((l) => !l.ok) : logs.value;
  return list.slice(-400).reverse();
});

let unlisten: Array<() => void> = [];
let offTauriDrop: (() => void) | null = null;
// 本次拖放过程中鼠标所在区域（由 DOM 事件记录，Tauri 的 payload 只给坐标）
const hoverZone = ref<"" | "src" | "out" | "preview">("");

const windowDrag = { x: 0, y: 0, active: false };
const lastDropAt = ref(0);

onMounted(async () => {
  // 记录最近一次拖动经过的区域
  window.addEventListener("dragenter", onWinDragEnter, true);
  window.addEventListener("dragover", onWinDragOver, true);
  window.addEventListener("dragleave", onWinDragLeave, true);
  window.addEventListener("drop", onWinDrop, true);
  // 阻止 webview 默认用文件导航
  window.addEventListener("dragover", (e) => e.preventDefault());
  window.addEventListener("drop", (e) => e.preventDefault());

  presets.value = await api.loadPresets().catch(() => []);

  try {
    offTauriDrop = await getCurrentWebview().onDragDropEvent((event) => {
      const p = event.payload;
      if (p.type === "enter" || p.type === "over") {
        windowDrag.active = true;
      } else if (p.type === "leave") {
        windowDrag.active = false;
        dragTarget.value = "";
        hoverZone.value = "";
      } else if (p.type === "drop") {
        windowDrag.active = false;
        handleDrop(p.paths || []);
      }
    });
  } catch {
    // 浏览器预览环境下没有 Tauri
  }

  unlisten = await Promise.all([
    events.onProgress((p) => Object.assign(progress, p)),
    events.onLog((l) => {
      logs.value.push(l);
      if (logs.value.length > 20000) logs.value.splice(0, 5000);
    }),
    events.onDone(onDone),
  ]);
});

onUnmounted(() => {
  unlisten.forEach((f) => f());
  offTauriDrop?.();
  window.removeEventListener("dragenter", onWinDragEnter, true);
  window.removeEventListener("dragover", onWinDragOver, true);
  window.removeEventListener("dragleave", onWinDragLeave, true);
  window.removeEventListener("drop", onWinDrop, true);
});

function onWinDragEnter(e: DragEvent) {
  e.preventDefault();
}
function onWinDragOver(e: DragEvent) {
  if (!windowDrag.active) return;
  e.preventDefault();
  // Tauri 的 drag event 里 position 是物理像素，这里改用 DOM 元素命中测试
  const el = document.elementFromPoint(e.clientX, e.clientY) as HTMLElement | null;
  const zoneEl = el?.closest("[data-drop-zone]") as HTMLElement | null;
  const zone = (zoneEl?.dataset.dropZone || "") as typeof hoverZone.value;
  hoverZone.value = zone;
  dragTarget.value = zone;
}
function onWinDragLeave() {
  // 窗口外时才清除
}
function onWinDrop() {
  lastDropAt.value = Date.now();
}

const dropPathsQueue = ref<string[]>([]);
function handleDrop(paths: string[]) {
  const zone = hoverZone.value;
  const target = zone || guessZone(paths);
  dragTarget.value = "";
  hoverZone.value = "";

  const imageExts = [".png", ".jpg", ".jpeg", ".webp"];
  const dirs = paths.filter((p) => !imageExts.some((e) => p.toLowerCase().endsWith(e)));
  const images = paths.filter((p) => imageExts.some((e) => p.toLowerCase().endsWith(e)));

  if (target === "out") {
    if (dirs.length) setOutDir(dirs[0]);
    else message.warning("请拖入文件夹作为输出目录");
    return;
  }
  if (target === "preview" || (target !== "src" && images.length && !dirs.length)) {
    if (images.length) loadSample(images[0]);
    else message.warning("请拖入一张 png / jpg / webp 图片");
    return;
  }
  if (dirs.length) {
    setSrcDir(dirs[0]);
  } else if (images.length) {
    loadSample(images[0]);
  }
}

function guessZone(_paths: string[]): typeof dragTarget.value {
  return "";
}

async function setSrcDir(dir: string) {
  config.srcDir = dir;
  srcCount.value = undefined;
  try {
    const n = await api.countImages(dir);
    srcCount.value = n;
    if (!n) {
      message.warning("该目录下没有找到可处理的图片");
      return;
    }
    await loadSample("");
  } catch (e) {
    message.error(String(e));
  }
}

async function setOutDir(dir: string) {
  config.outDir = dir;
  if (config.srcDir && dir && dir.replace(/[\\/]+$/, "") === config.srcDir.replace(/[\\/]+$/, "")) {
    dialog.error({
      title: "目录冲突",
      content: "素材目录与输出目录不能相同，请重新选择输出目录。",
    });
  }
}

async function loadSample(path: string) {
  try {
    previewing.value = true;
    const info = path ? await api.filePreview(path) : await api.pickRandomPreview(config.srcDir);
    if (!info) {
      originalPreview.value = null;
      croppedPreview.value = "";
      return;
    }
    originalPreview.value = info;
    samplePath.value = info.absPath;
    await refreshPreview();
  } catch (e) {
    message.error(String(e));
  } finally {
    previewing.value = false;
  }
}

async function refreshPreview() {
  if (!samplePath.value) {
    if (config.srcDir) await loadSample("");
    return;
  }
  if (!config.targetW || !config.targetH) {
    message.warning("请填写目标宽高");
    return;
  }
  try {
    previewing.value = true;
    const [, cropped] = await Promise.all([
      updateCropBox(),
      api.renderPreview(samplePath.value, { ...config }),
    ]);
    croppedPreview.value = cropped;
    dirty.value = false;
  } catch (e) {
    message.error(String(e));
  } finally {
    previewing.value = false;
  }
}

async function updateCropBox() {
  if (!originalPreview.value || !config.targetW || !config.targetH) {
    cropBox.value = null;
    return;
  }
  if (config.fill.kind === "stretch") {
    cropBox.value = { x: 0, y: 0, w: 1, h: 1 };
    return;
  }
  const info = originalPreview.value;
  const [x, y, w, h] = await api.cropRect(
    info.width,
    info.height,
    config.targetW,
    config.targetH,
    config.mode,
    config.allowUpscale
  );
  cropBox.value = {
    x: x / info.width,
    y: y / info.height,
    w: w / info.width,
    h: h / info.height,
  };
}

watch(
  () => [
    config.targetW,
    config.targetH,
    config.mode,
    config.allowUpscale,
    JSON.stringify(config.fill),
    config.format,
    config.quality,
  ],
  () => {
    dirty.value = true;
    if (autoPreview.value) refreshPreview();
  }
);

async function savePresets(list: Preset[]) {
  presets.value = list;
  try {
    await api.savePresets(list);
  } catch (e) {
    message.error(String(e));
  }
}

function validate(): string | null {
  if (!config.srcDir) return "请选择素材目录";
  if (!config.outDir) return "请选择输出目录";
  if (!config.targetW || !config.targetH) return "请填写目标宽高";
  const norm = (s: string) => s.replace(/[\\/]+$/, "");
  if (norm(config.srcDir) === norm(config.outDir)) return "素材目录与输出目录不能相同";
  return null;
}

async function startExport() {
  const err = validate();
  if (err) {
    message.warning(err);
    return;
  }
  logs.value = [];
  Object.assign(progress, { taskId: 0, total: 0, done: 0, failed: 0, current: "" });
  try {
    running.value = true;
    await api.startExport({ ...config });
  } catch (e) {
    running.value = false;
    dialog.error({ title: "无法开始导出", content: String(e) });
  }
}

async function cancelExport() {
  await api.cancelExport();
  message.info("正在取消，当前批次处理完后停止…");
}

function onDone(p: DonePayload) {
  running.value = false;
  Object.assign(progress, {
    total: p.total,
    done: p.done,
    failed: p.failed,
  });
  if (p.cancelled) {
    message.info(`任务已取消：完成 ${p.done} 张，失败 ${p.failed} 张`);
    return;
  }
  dialog.success({
    title: "导出完成",
    content: `共 ${p.total} 张，成功 ${p.done - p.failed} 张，失败 ${p.failed} 张。`,
    positiveText: "打开输出文件夹",
    negativeText: "关闭",
    onPositiveClick: () => api.revealInDir(p.outDir),
  });
}

function openOutput() {
  if (config.outDir) api.revealInDir(config.outDir);
}

const previewMeta = computed(() => {
  const i = originalPreview.value;
  if (!i) return "";
  return `${i.width} × ${i.height} px`;
});

/** 裁切框保留的原图像素尺寸，帮助理解「框 = 保留范围」而非输出尺寸 */
const cropLabel = computed(() => {
  const b = cropBox.value;
  const i = originalPreview.value;
  if (!b || !i) return "";
  return `保留 ${Math.round(b.w * i.width)} × ${Math.round(b.h * i.height)} px`;
});

const fillLabel = computed(() => {
  const f = config.fill;
  if (f.kind === "transparent") return "透明填充";
  if (f.kind === "stretch") return "拉伸铺满";
  return `背景色 ${f.hex.toUpperCase()}`;
});

const ratioLabel = computed(() => {
  if (!config.targetW || !config.targetH) return "";
  const g = (a: number, b: number): number => (b ? g(b, a % b) : a);
  const d = g(config.targetW, config.targetH) || 1;
  const w = config.targetW / d;
  const h = config.targetH / d;
  const ratio = config.targetW / config.targetH;
  return w <= 40 && h <= 40
    ? `${w}:${h}`
    : `${ratio.toFixed(2)}:1`;
});
</script>

<template>
  <div class="app">
    <!-- 顶部：目录 -->
    <section class="top">
      <div class="top-grid" data-drop-zone="src">
        <dir-picker
          v-model="config.srcDir"
          label="素材目录"
          :image-count="srcCount"
          :dragging="dragTarget === 'src'"
          @picked="setSrcDir"
        />
        <div data-drop-zone="out">
          <dir-picker
            v-model="config.outDir"
            label="输出目录"
            placeholder="点击浏览或拖拽文件夹到此处"
            :dragging="dragTarget === 'out'"
            @picked="setOutDir"
          />
        </div>
      </div>
      <div class="tips-row">
        <div class="tips">
          支持 png / jpg / jpeg / webp，自动递归子目录 · 纯本地处理，不上传任何图片
        </div>
        <div class="theme-toggle">
          <span class="dim">深色模式</span>
          <n-switch v-model:value="isDark" size="small" />
        </div>
      </div>
    </section>

    <!-- 主体 + 底部一起滚动，窗口变小时可纵向滚动 -->
    <div class="scroll-body scroll-y">
      <!-- 主体：参数 + 预览 -->
      <section class="main">
        <aside class="side">
          <param-panel
            :config="config"
            :presets="presets"
            :disabled="running"
            @update:config="(v) => Object.assign(config, v)"
            @save-presets="savePresets"
            @apply="refreshPreview"
          />
        </aside>

        <div class="content">
          <div class="content-head">
            <div class="summary">
              <n-tag :bordered="false" type="primary">居中裁切</n-tag>
              <span class="dim">
                {{ config.targetW || "-" }} × {{ config.targetH || "-" }} px
                <template v-if="ratioLabel">（{{ ratioLabel }}）</template>
              </span>
              <span class="dim">· {{ fillLabel }}</span>
              <n-tag size="small" :bordered="false">{{ config.format.toUpperCase() }}</n-tag>
              <n-tag v-if="dirty && !autoPreview" size="small" type="warning" :bordered="false">
                参数已修改，点击「应用并预览」
              </n-tag>
            </div>
            <div class="head-right">
              <span class="dim">自动预览</span>
              <n-switch v-model:value="autoPreview" size="small" />
              <n-button
                size="small"
                secondary
                :loading="previewing"
                :disabled="!config.srcDir"
                @click="loadSample('')"
              >
                换一张样本
              </n-button>
            </div>
          </div>

          <!-- 双栏预览：窄窗口时改为上下堆叠 -->
          <div class="previews">
            <div class="pane-wrap" data-drop-zone="preview">
              <preview-pane
                title="原图"
                :src="originalPreview?.original"
                :meta="previewMeta"
                :crop-box="cropBox"
                :crop-label="cropLabel"
                empty-text="选择素材目录后自动加载样例，或拖入单张图片测试"
                @drop-file="loadSample"
              />
            </div>
            <div class="divider" />
            <div class="pane-wrap">
              <preview-pane
                title="裁切后"
                :src="croppedPreview"
                :meta="`${config.targetW || '-'} × ${config.targetH || '-'}`"
                highlight
                empty-text="点击「应用并预览」查看效果"
              />
            </div>
          </div>
        </div>
      </section>

      <!-- 底部：执行面板 -->
      <section class="bottom">
      <div class="actions">
        <n-button
          type="primary"
          size="large"
          :disabled="running"
          :loading="running && !progress.done"
          @click="startExport"
        >
          <template #icon><n-icon><play-outline /></n-icon></template>
          开始批量导出
        </n-button>
        <n-button size="large" :disabled="!running" @click="cancelExport">
          <template #icon><n-icon><stop-outline /></n-icon></template>
          取消任务
        </n-button>
        <n-button size="large" quaternary :disabled="!config.outDir" @click="openOutput">
          <template #icon><n-icon><folder-open-outline /></n-icon></template>
          打开输出目录
        </n-button>
      </div>

      <div class="progress-row">
        <n-progress
          type="line"
          :percentage="percent"
          :show-indicator="false"
          :height="8"
          :status="progress.failed && !running ? 'warning' : 'default'"
          style="flex: 1"
        />
        <div class="stats">
          <span>总数 <b>{{ progress.total }}</b></span>
          <span>已完成 <b>{{ progress.done }}</b></span>
          <span :class="{ danger: failCount > 0 }">失败 <b>{{ failCount }}</b></span>
          <span class="pct">{{ percent }}%</span>
        </div>
      </div>

      <div class="logs">
        <div class="logs-head">
          <span class="logs-title">日志</span>
          <n-tag size="small" :bordered="false" type="success">
            <template #icon><n-icon><checkmark-circle-outline /></n-icon></template>
            {{ progress.done - progress.failed }}
          </n-tag>
          <n-tag size="small" :bordered="false" type="error">
            <template #icon><n-icon><close-circle-outline /></n-icon></template>
            {{ failCount }}
          </n-tag>
          <n-switch v-model:value="onlyFailed" size="small">
            <template #checked>仅看失败</template>
            <template #unchecked>全部</template>
          </n-switch>
          <span class="logs-hint">最新在上，仅显示最近 400 条</span>
        </div>
        <n-scrollbar class="log-scroll">
          <div v-if="!visibleLogs.length" class="log-empty">暂无日志</div>
          <div
            v-for="(l, i) in visibleLogs"
            :key="i"
            class="log-item"
            :class="{ fail: !l.ok }"
          >
            <n-icon class="li-icon">
              <checkmark-circle-outline v-if="l.ok" />
              <close-circle-outline v-else />
            </n-icon>
            <span class="li-name">{{ l.name }}</span>
            <span v-if="l.reason" class="li-reason">{{ l.reason }}</span>
          </div>
        </n-scrollbar>
      </div>
      </section>
    </div>
  </div>
</template>

<style scoped>
.app {
  display: flex;
  flex-direction: column;
  height: 100%;
  min-width: 0;
  background: var(--bg);
  overflow: hidden;
}
.top {
  background: var(--surface);
  padding: 12px 16px 10px;
  border-bottom: 1px solid var(--border);
  flex: 0 0 auto;
}
.top-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 20px;
}
.tips-row {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 8px;
}
.theme-toggle {
  display: flex;
  align-items: center;
  gap: 6px;
  flex: 0 0 auto;
}
.tips {
  flex: 1;
  font-size: 12px;
  color: var(--text-4);
}
.scroll-body {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  overflow-y: auto;
}
.main {
  flex: 0 0 auto;
  display: flex;
  min-height: 520px;
  padding: 12px;
  gap: 12px;
}
.side {
  width: 300px;
  flex: 0 0 300px;
  background: var(--surface);
  border-radius: 10px;
  border: 1px solid var(--border);
  min-height: 0;
}
.content {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  background: var(--surface);
  border-radius: 10px;
  border: 1px solid var(--border);
  min-height: 0;
  overflow: hidden;
}
.content-head {
  display: flex;
  align-items: center;
  padding: 8px 12px;
  border-bottom: 1px solid var(--border);
  gap: 12px;
}
.summary {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  flex-wrap: wrap;
}
.dim {
  color: var(--text-3);
  font-size: 12px;
}
.head-right {
  margin-left: auto;
  display: flex;
  align-items: center;
  gap: 8px;
}
.previews {
  flex: 1;
  display: flex;
  min-height: 0;
}
.pane-wrap {
  flex: 1;
  min-width: 0;
  display: flex;
  padding: 0;
}
.divider {
  width: 1px;
  background: var(--border);
  flex: 0 0 1px;
}

/* 窄窗口：参数区与预览改为上下堆叠，预览双栏改为上下两行 */
@media (max-width: 1180px) {
  .main {
    flex-direction: column;
    min-height: 0;
  }
  .side {
    width: 100%;
    flex: 0 0 auto;
    max-height: none;
  }
  .content {
    min-height: 480px;
  }
}
@media (max-width: 860px) {
  .top-grid {
    grid-template-columns: 1fr;
    gap: 12px;
  }
  .previews {
    flex-direction: column;
  }
  .previews .divider {
    width: auto;
    height: 1px;
    flex: 0 0 1px;
  }
  .pane-wrap {
    min-height: 260px;
  }
  .actions {
    flex-wrap: wrap;
  }
  .progress-row {
    flex-direction: column;
    align-items: stretch;
    gap: 8px;
  }
  .stats {
    justify-content: space-between;
  }
}
.bottom {
  background: var(--surface);
  border-top: 1px solid var(--border);
  padding: 12px 16px;
  display: flex;
  flex-direction: column;
  gap: 10px;
  flex: 0 0 auto;
}
.actions {
  display: flex;
  gap: 10px;
}
.progress-row {
  display: flex;
  align-items: center;
  gap: 16px;
}
.stats {
  display: flex;
  gap: 14px;
  font-size: 12px;
  color: var(--text-3);
  white-space: nowrap;
}
.stats b {
  color: var(--text);
  font-weight: 600;
}
.stats .danger b {
  color: #e5484d;
}
.pct {
  font-family: ui-monospace, monospace;
  color: #4c6ef5;
  font-weight: 600;
}
.logs {
  border: 1px solid var(--border);
  border-radius: 8px;
  overflow: hidden;
}
.logs-head {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 6px 10px;
  background: var(--surface-2);
  border-bottom: 1px solid var(--border-2);
}
.logs-title {
  font-weight: 600;
  font-size: 13px;
}
.logs-hint {
  margin-left: auto;
  font-size: 11px;
  color: var(--text-4);
}
.log-scroll {
  height: 132px;
}
.log-empty {
  padding: 16px;
  text-align: center;
  color: var(--text-4);
  font-size: 12px;
}
.log-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 3px 10px;
  font-size: 12px;
  font-family: ui-monospace, SFMono-Regular, monospace;
}
.log-item:hover {
  background: var(--hover);
}
.li-icon {
  color: #18a058;
}
.log-item.fail .li-icon {
  color: #e5484d;
}
.li-name {
  color: var(--text-2);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.log-item.fail .li-name {
  color: #e5484d;
}
.li-reason {
  color: #b06000;
  margin-left: auto;
  white-space: nowrap;
}
</style>

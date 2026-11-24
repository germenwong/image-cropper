<script setup lang="ts">
import { computed, ref } from "vue";
import { NButton, NIcon, NTag } from "naive-ui";
import { AddOutline, RemoveOutline, ScanOutline } from "@vicons/ionicons5";

const props = defineProps<{
  title: string;
  src?: string;
  meta?: string;
  highlight?: boolean;
  emptyText?: string;
  /** 裁切框，单位为相对原图的比例 0~1：{x,y,w,h} */
  cropBox?: { x: number; y: number; w: number; h: number } | null;
}>();
const emit = defineEmits<{ (e: "drop-file", path: string): void }>();

const scale = ref(1);
const tx = ref(0);
const ty = ref(0);
const dragging = ref(false);
const isFileDragOver = ref(false);
let startX = 0;
let startY = 0;
let baseX = 0;
let baseY = 0;

const stage = ref<HTMLDivElement | null>(null);

function onWheel(e: WheelEvent) {
  if (!props.src) return;
  e.preventDefault();
  const factor = e.deltaY < 0 ? 1.12 : 1 / 1.12;
  scale.value = Math.min(8, Math.max(0.1, scale.value * factor));
}

function onPointerDown(e: PointerEvent) {
  if (!props.src) return;
  dragging.value = true;
  startX = e.clientX;
  startY = e.clientY;
  baseX = tx.value;
  baseY = ty.value;
  (e.target as HTMLElement).setPointerCapture(e.pointerId);
}

function onPointerMove(e: PointerEvent) {
  if (!dragging.value) return;
  tx.value = baseX + (e.clientX - startX);
  ty.value = baseY + (e.clientY - startY);
}

function onPointerUp() {
  dragging.value = false;
}

function reset() {
  scale.value = 1;
  tx.value = 0;
  ty.value = 0;
}

const transformStyle = computed(() => ({
  transform: `translate(${tx.value}px, ${ty.value}px) scale(${scale.value})`,
  cursor: dragging.value ? "grabbing" : "grab",
}));

// ---- 拖拽单张图片进来测试 ----
async function onDragOver(e: DragEvent) {
  e.preventDefault();
  if (e.dataTransfer?.types.includes("Files")) {
    isFileDragOver.value = true;
  }
}
function onDragLeave() {
  isFileDragOver.value = false;
}
function onDrop(e: DragEvent) {
  e.preventDefault();
  isFileDragOver.value = false;
  const dt = e.dataTransfer;
  if (!dt) return;
  // Tauri 拦截文件拖放时 dataTransfer 中文件不可用，由主窗口事件统一处理
  const f = dt.files?.[0] as any;
  const p = f?.path;
  if (typeof p === "string") emit("drop-file", p);
}

defineExpose({ reset });
</script>

<template>
  <div class="pane">
    <div class="head">
      <span class="title">{{ title }}</span>
      <n-tag v-if="meta" size="small" :bordered="false">{{ meta }}</n-tag>
      <div class="tools">
        <n-button size="tiny" quaternary circle title="缩小" @click="scale = Math.max(0.1, scale / 1.2)">
          <template #icon><n-icon><remove-outline /></n-icon></template>
        </n-button>
        <span class="pct">{{ Math.round(scale * 100) }}%</span>
        <n-button size="tiny" quaternary circle title="放大" @click="scale = Math.min(8, scale * 1.2)">
          <template #icon><n-icon><add-outline /></n-icon></template>
        </n-button>
        <n-button size="tiny" quaternary circle title="复位（双击画布）" @click="reset">
          <template #icon><n-icon><scan-outline /></n-icon></template>
        </n-button>
      </div>
    </div>
    <div
      ref="stage"
      class="stage"
      :class="{ 'drag-over': isFileDragOver, hl: highlight }"
      @wheel="onWheel"
      @pointerdown="onPointerDown"
      @pointermove="onPointerMove"
      @pointerup="onPointerUp"
      @pointercancel="onPointerUp"
      @dblclick="reset"
      @dragover="onDragOver"
      @dragleave="onDragLeave"
      @drop="onDrop"
    >
      <div v-if="src" class="img-wrap" :style="transformStyle">
        <div class="img-inner">
          <img :src="src" draggable="false" />
          <div
            v-if="cropBox"
            class="crop-box"
            :style="{
              left: cropBox.x * 100 + '%',
              top: cropBox.y * 100 + '%',
              width: cropBox.w * 100 + '%',
              height: cropBox.h * 100 + '%',
            }"
          >
            <span class="tag">裁切区域</span>
          </div>
        </div>
      </div>
      <div v-else class="empty">
        {{ emptyText || "暂无预览" }}
      </div>
      <div v-if="isFileDragOver" class="drop-hint">松开以加载该图片</div>
    </div>
  </div>
</template>

<style scoped>
.pane {
  display: flex;
  flex-direction: column;
  min-width: 0;
  min-height: 0;
  height: 100%;
}
.head {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 10px;
  border-bottom: 1px solid #eceef4;
}
.title {
  font-weight: 600;
  font-size: 13px;
  color: #4a5160;
}
.tools {
  margin-left: auto;
  display: flex;
  align-items: center;
  gap: 2px;
}
.pct {
  font-size: 11px;
  color: #8a90a0;
  width: 40px;
  text-align: center;
  font-family: ui-monospace, monospace;
}
.stage {
  position: relative;
  flex: 1;
  min-height: 180px;
  overflow: hidden;
  display: flex;
  align-items: center;
  justify-content: center;
  background: #f7f8fa;
  border: 1px solid transparent;
  touch-action: none;
}
.stage.hl {
  background: #fbfbff;
}
.stage.drag-over {
  border-color: #4c6ef5;
  background: #eef1ff;
}
.img-wrap {
  will-change: transform;
  transition: transform 0.06s linear;
}
.img-inner {
  position: relative;
  display: inline-block;
}
.crop-box {
  position: absolute;
  border: 2px solid #4c6ef5;
  box-shadow: 0 0 0 9999px rgba(15, 22, 45, 0.42);
  pointer-events: none;
}
.crop-box .tag {
  position: absolute;
  top: -22px;
  left: -2px;
  background: #4c6ef5;
  color: #fff;
  font-size: 11px;
  padding: 1px 6px;
  border-radius: 3px 3px 3px 0;
  white-space: nowrap;
}
.stage :deep(img) {
  display: block;
  max-width: 100%;
  max-height: 100%;
  box-shadow: 0 2px 12px rgba(20, 30, 60, 0.12);
  background-color: #fff;
  background-image: linear-gradient(45deg, #e9ebf0 25%, transparent 25%),
    linear-gradient(-45deg, #e9ebf0 25%, transparent 25%),
    linear-gradient(45deg, transparent 75%, #e9ebf0 75%),
    linear-gradient(-45deg, transparent 75%, #e9ebf0 75%);
  background-size: 16px 16px;
  background-position: 0 0, 0 8px, 8px -8px, -8px 0;
}
.empty {
  color: #a0a5b3;
  font-size: 13px;
}
.drop-hint {
  position: absolute;
  bottom: 14px;
  font-size: 12px;
  color: #4c6ef5;
  background: #fff;
  padding: 4px 10px;
  border-radius: 20px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}
</style>

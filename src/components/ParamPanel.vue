<script setup lang="ts">
import { computed, ref } from "vue";
import {
  NButton,
  NCheckbox,
  NColorPicker,
  NDivider,
  NInputGroup,
  NInputNumber,
  NPopconfirm,
  NRadio,
  NRadioGroup,
  NSelect,
  NSlider,
  NTag,
  NInput,
  NIcon,
  NTooltip,
  useMessage,
} from "naive-ui";
import { AddOutline, CloseOutline, FlashOutline } from "@vicons/ionicons5";
import type { CropConfig, CropMode, FillMode, OutputFormat, Preset } from "../types";

const props = defineProps<{
  config: CropConfig;
  presets: Preset[];
  disabled?: boolean;
}>();
const emit = defineEmits<{
  (e: "update:config", v: CropConfig): void;
  (e: "save-presets", v: Preset[]): void;
  (e: "apply"): void;
}>();

const message = useMessage();
const newName = ref("");
const showPresetForm = ref(false);

const cfg = computed(() => props.config);

function patch(p: Partial<CropConfig>) {
  emit("update:config", { ...props.config, ...p });
}

const fillKind = computed(() => props.config.fill.kind);
const fillColor = computed(() =>
  props.config.fill.kind === "color" ? props.config.fill.hex : "#ffffff"
);

function setFillKind(kind: FillMode["kind"]) {
  if (kind === "color") {
    patch({ fill: { kind: "color", hex: fillColor.value } });
  } else {
    patch({ fill: { kind } as FillMode });
  }
}

function setFillColor(hex: string) {
  patch({ fill: { kind: "color", hex } });
}

const formatOptions = [
  { label: "PNG（无损，支持透明）", value: "png" },
  { label: "JPG（有损，透明铺白）", value: "jpg" },
  { label: "WebP（无损）", value: "webp" },
];

function applyPreset(p: Preset) {
  patch({ targetW: p.w, targetH: p.h });
  message.success(`已填充 ${p.name}：${p.w} × ${p.h}`);
}

function addPreset() {
  const w = props.config.targetW;
  const h = props.config.targetH;
  if (!w || !h) {
    message.warning("请先填写目标宽高");
    return;
  }
  const name = newName.value.trim() || `${w}×${h}`;
  const list = [
    ...props.presets,
    { id: String(Date.now()), name, w, h },
  ];
  emit("save-presets", list);
  newName.value = "";
  showPresetForm.value = false;
  message.success("预设已保存");
}

function removePreset(id: string) {
  emit(
    "save-presets",
    props.presets.filter((p) => p.id !== id)
  );
}

const qualityDisabled = computed(() => props.config.format === "png");
</script>

<template>
  <div class="panel">
    <div class="block">
      <div class="block-title">
        模式
        <n-tooltip>
          <template #trigger>
            <span class="hint">?</span>
          </template>
          规格输出：目标尺寸决定取景比例，再缩放到该像素（主流做法）<br />
          像素裁切：从原图按目标像素 1:1 截取，不缩放
        </n-tooltip>
      </div>
      <n-radio-group
        :value="cfg.mode"
        size="small"
        :disabled="disabled"
        @update:value="(v: CropMode) => patch({ mode: v })"
      >
        <n-radio value="spec">规格输出</n-radio>
        <n-radio value="pixel">像素裁切</n-radio>
      </n-radio-group>
      <div class="mode-hint">
        {{
          cfg.mode === "spec"
            ? "按目标比例取景，再缩放到目标像素"
            : `从原图中心截取 ${cfg.targetW || "-"} × ${cfg.targetH || "-"} px，不缩放`
        }}
      </div>
    </div>

    <div class="block">
      <div class="block-title">尺寸</div>
      <n-input-group>
        <n-input-number
          v-model:value="cfg.targetW"
          :min="1"
          :max="20000"
          :show-button="false"
          placeholder="宽"
          style="width: 110px"
          :disabled="disabled"
          @update:value="(v) => patch({ targetW: v ?? 0 })"
        >
          <template #suffix>px</template>
        </n-input-number>
        <div class="times">×</div>
        <n-input-number
          v-model:value="cfg.targetH"
          :min="1"
          :max="20000"
          :show-button="false"
          placeholder="高"
          style="width: 110px"
          :disabled="disabled"
          @update:value="(v) => patch({ targetH: v ?? 0 })"
        >
          <template #suffix>px</template>
        </n-input-number>
      </n-input-group>
    </div>

    <div class="block">
      <div class="block-title">对齐方式</div>
      <n-tag :bordered="false" type="success" size="small">居中裁切（MVP 固定）</n-tag>
    </div>

    <div class="block">
      <div class="block-title">
        原图小于目标画布
        <n-tooltip>
          <template #trigger>
            <span class="hint">?</span>
          </template>
          原图小于目标尺寸时默认不放大，居中放置并用下方方式填充空余区域；勾选「允许放大」后改为放大铺满（画质会下降）
        </n-tooltip>
      </div>
      <n-radio-group
        :value="fillKind"
        size="small"
        :disabled="disabled"
        @update:value="setFillKind"
      >
        <n-radio value="transparent">透明</n-radio>
        <n-radio value="color">纯色</n-radio>
        <n-radio value="stretch">拉伸铺满</n-radio>
      </n-radio-group>

      <div v-if="fillKind === 'color'" class="color-row">
        <n-color-picker
          :value="fillColor"
          :show-alpha="true"
          size="small"
          :modes="['hex']"
          @update:value="setFillColor"
        />
        <span class="hex">{{ fillColor.toUpperCase() }}</span>
        <div class="quick">
          <span class="sw" style="background: #fff" @click="setFillColor('#ffffff')" />
          <span class="sw" style="background: #000" @click="setFillColor('#000000')" />
          <span class="sw" style="background: #f5f5f5" @click="setFillColor('#f5f5f5')" />
        </div>
      </div>
      <div v-if="fillKind === 'stretch'" class="warn">
        拉伸会改变图片比例，仅在明确需要时开启
      </div>
      <n-checkbox
        v-if="cfg.mode === 'spec'"
        :checked="cfg.allowUpscale"
        :disabled="disabled || fillKind === 'stretch'"
        size="small"
        class="upscale"
        @update:checked="(v: boolean) => patch({ allowUpscale: v })"
      >
        允许放大原图（画质会下降）
      </n-checkbox>
      <div v-else class="mode-hint">像素裁切不做放大，原图不足时按上方方式填充</div>
    </div>

    <div class="block">
      <div class="block-title">输出格式</div>
      <n-select
        :value="cfg.format"
        :options="formatOptions"
        size="small"
        :disabled="disabled"
        @update:value="(v: OutputFormat) => patch({ format: v })"
      />
    </div>

    <div class="block">
      <div class="block-title">
        输出质量
        <span class="q">{{ qualityDisabled ? "PNG 忽略质量" : cfg.quality }}</span>
      </div>
      <n-slider
        :value="cfg.quality"
        :step="1"
        :min="1"
        :max="100"
        :disabled="qualityDisabled || disabled"
        @update:value="(v: number) => patch({ quality: v })"
      />
    </div>

    <n-divider style="margin: 10px 0" />

    <div class="block">
      <div class="block-title">
        尺寸预设
        <n-button text size="tiny" type="primary" @click="showPresetForm = !showPresetForm">
          <template #icon><n-icon><add-outline /></n-icon></template>
          从当前尺寸新增
        </n-button>
      </div>
      <div v-if="showPresetForm" class="preset-form">
        <n-input
          v-model:value="newName"
          size="small"
          placeholder="预设名称，如 头像 / banner"
          @keyup.enter="addPreset"
        />
        <n-button size="small" type="primary" @click="addPreset">保存</n-button>
      </div>
      <div class="presets">
        <div v-for="p in presets" :key="p.id" class="preset">
          <div class="preset-main" @click="applyPreset(p)">
            <n-icon class="flash"><flash-outline /></n-icon>
            <span class="pname">{{ p.name }}</span>
            <span class="psize">{{ p.w }}×{{ p.h }}</span>
          </div>
          <n-popconfirm @positive-click="removePreset(p.id)">
            <template #trigger>
              <n-button size="tiny" quaternary circle>
                <template #icon><n-icon><close-outline /></n-icon></template>
              </n-button>
            </template>
            删除该预设？
          </n-popconfirm>
        </div>
        <div v-if="!presets.length" class="empty">暂无预设，点击上方新增</div>
      </div>
    </div>

    <div class="apply-wrap">
      <n-button type="primary" block :disabled="disabled" @click="emit('apply')">
        应用并预览
      </n-button>
    </div>
  </div>
</template>

<style scoped>
.panel {
  padding: 14px 16px;
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow-y: auto;
}
@media (max-width: 1180px) {
  .panel {
    height: auto;
    overflow-y: visible;
  }
}
.block {
  margin-bottom: 14px;
}
.block-title {
  display: flex;
  align-items: center;
  gap: 6px;
  font-weight: 600;
  color: #4a5160;
  font-size: 13px;
  margin-bottom: 8px;
}
.times {
  padding: 0 8px;
  display: flex;
  align-items: center;
  color: #8a90a0;
}
.hint {
  width: 15px;
  height: 15px;
  border-radius: 50%;
  background: #e4e7ee;
  color: #7a8090;
  font-size: 10px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  cursor: help;
}
.q {
  margin-left: auto;
  font-weight: 500;
  color: #4c6ef5;
}
.color-row {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 8px;
}
.hex {
  font-family: ui-monospace, monospace;
  font-size: 12px;
  color: #6b7280;
}
.quick {
  margin-left: auto;
  display: flex;
  gap: 4px;
}
.sw {
  width: 18px;
  height: 18px;
  border-radius: 4px;
  border: 1px solid #dfe2e8;
  cursor: pointer;
}
.mode-hint {
  margin-top: 6px;
  font-size: 12px;
  color: #8a90a0;
  line-height: 1.5;
}
.upscale {
  margin-top: 10px;
}
.warn {
  margin-top: 8px;
  font-size: 12px;
  color: #d9822b;
  background: #fff8ec;
  border-radius: 4px;
  padding: 4px 8px;
}
.preset-form {
  display: flex;
  gap: 6px;
  margin-bottom: 8px;
}
.presets {
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.preset {
  display: flex;
  align-items: center;
  border: 1px solid #eaecf2;
  border-radius: 6px;
  padding: 4px 4px 4px 8px;
  background: #fbfcfe;
}
.preset:hover {
  border-color: #c7d2fe;
  background: #f5f7ff;
}
.preset-main {
  display: flex;
  align-items: center;
  gap: 8px;
  flex: 1;
  cursor: pointer;
  min-width: 0;
}
.flash {
  color: #f0a020;
}
.pname {
  font-size: 13px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.psize {
  margin-left: auto;
  font-size: 12px;
  color: #8a90a0;
  font-family: ui-monospace, monospace;
}
.empty {
  font-size: 12px;
  color: #a0a5b3;
  text-align: center;
  padding: 8px 0;
}
.apply-wrap {
  margin-top: auto;
  padding-top: 12px;
}
</style>

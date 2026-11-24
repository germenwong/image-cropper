<script setup lang="ts">
import { ref, watch } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import {
  NButton,
  NInput,
  NIcon,
  NTag,
  NTooltip,
  useMessage,
} from "naive-ui";
import { FolderOpenOutline } from "@vicons/ionicons5";

const props = defineProps<{
  modelValue: string;
  label: string;
  placeholder?: string;
  imageCount?: number;
  dragging?: boolean;
}>();

const emit = defineEmits<{
  (e: "update:modelValue", v: string): void;
  (e: "picked", v: string): void;
}>();

const message = useMessage();
const inputValue = ref(props.modelValue);

watch(
  () => props.modelValue,
  (v) => (inputValue.value = v)
);

async function pick() {
  try {
    const res = await open({ directory: true, multiple: false, title: props.label });
    if (typeof res === "string") {
      emit("update:modelValue", res);
      emit("picked", res);
    }
  } catch (e) {
    message.error(String(e));
  }
}

function commit(v: string) {
  emit("update:modelValue", v.trim());
  emit("picked", v.trim());
}
</script>

<template>
  <div class="picker" :class="{ dragging }">
    <div class="label">
      {{ label }}
      <n-tag v-if="imageCount !== undefined" size="small" :bordered="false" type="info">
        {{ imageCount }} 张图片
      </n-tag>
    </div>
    <div class="row">
      <n-input
        v-model:value="inputValue"
        :placeholder="placeholder || '点击浏览或拖拽文件夹到此处'"
        @blur="commit(inputValue)"
        @keyup.enter="commit(inputValue)"
      />
      <n-tooltip trigger="hover">
        <template #trigger>
          <n-button secondary @click="pick">
            <template #icon><n-icon><folder-open-outline /></n-icon></template>
            浏览
          </n-button>
        </template>
        选择文件夹
      </n-tooltip>
    </div>
  </div>
</template>

<style scoped>
.picker {
  transition: 0.15s;
  border: 1px dashed transparent;
  border-radius: 8px;
  padding: 4px;
  margin: -4px;
}
.picker.dragging {
  border-color: #4c6ef5;
  background: var(--drop-bg);
}
.label {
  display: flex;
  align-items: center;
  gap: 8px;
  font-weight: 600;
  color: var(--text-2);
  margin-bottom: 6px;
}
.row {
  display: flex;
  gap: 8px;
}
.row :deep(.n-input) {
  flex: 1;
}
</style>

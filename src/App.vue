<script setup lang="ts">
import {
  NConfigProvider,
  NDialogProvider,
  NMessageProvider,
  NNotificationProvider,
  darkTheme,
  zhCN,
  dateZhCN,
} from "naive-ui";
import MainView from "./views/MainView.vue";
import { isDark } from "./theme";

const themeOverrides = {
  common: {
    primaryColor: "#4c6ef5",
    primaryColorHover: "#5c7cfa",
    primaryColorPressed: "#4263eb",
    borderRadius: "6px",
  },
};
</script>

<template>
  <n-config-provider
    class="root-provider"
    :locale="zhCN"
    :date-locale="dateZhCN"
    :theme="isDark ? darkTheme : null"
    :theme-overrides="themeOverrides"
  >
    <n-message-provider>
      <n-dialog-provider>
        <n-notification-provider>
          <main-view />
        </n-notification-provider>
      </n-dialog-provider>
    </n-message-provider>
  </n-config-provider>
</template>

<style>
/* Naive UI 的 provider 默认是 block，需限制成撑满且可收缩的容器 */
.root-provider,
.root-provider > .n-message-provider,
.root-provider > .n-message-provider > .n-dialog-provider,
.root-provider > .n-message-provider > .n-dialog-provider > .n-notification-provider {
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: 0;
  flex: 1 1 auto;
}
.root-provider > .n-message-provider > .n-dialog-provider > .n-notification-provider > * {
  flex: 1 1 auto;
  min-height: 0;
}

/* 弹窗遮罩毛玻璃 */
.n-modal-mask {
  backdrop-filter: blur(8px);
}
</style>

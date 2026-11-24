<script setup lang="ts">
import {
  NConfigProvider,
  NDialogProvider,
  NMessageProvider,
  NNotificationProvider,
  zhCN,
  dateZhCN,
} from "naive-ui";
import MainView from "./views/MainView.vue";

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
  <n-config-provider :locale="zhCN" :date-locale="dateZhCN" :theme-overrides="themeOverrides">
    <n-message-provider>
      <n-dialog-provider>
        <n-notification-provider>
          <main-view />
        </n-notification-provider>
      </n-dialog-provider>
    </n-message-provider>
  </n-config-provider>
</template>

import { ref, watch } from "vue";

const STORAGE_KEY = "ui-theme";

const saved = localStorage.getItem(STORAGE_KEY);
export const isDark = ref(saved === "dark");

watch(
  isDark,
  (dark) => {
    localStorage.setItem(STORAGE_KEY, dark ? "dark" : "light");
    document.documentElement.dataset.theme = dark ? "dark" : "light";
  },
  { immediate: true }
);

export function toggleTheme() {
  isDark.value = !isDark.value;
}

#!/usr/bin/env node
// 一键启动本地调试：检查环境 → 装依赖 → 起 Tauri 开发模式（Vite 1420 + 桌面窗口）
import { spawnSync } from "node:child_process";
import {
  ROOT,
  checkNodeVersion,
  die,
  ensureCargo,
  ensureDeps,
  envWithCargo,
  isWin,
  npmCmd,
  run,
} from "./env.mjs";

checkNodeVersion();

const { pathDir } = ensureCargo();

if (process.platform === "darwin") {
  const clt = spawnSync("xcode-select", ["-p"], { stdio: "ignore" });
  if (clt.status !== 0) {
    die("缺少 Xcode Command Line Tools，请先执行：xcode-select --install");
  }
}

ensureDeps();

console.log("→ 启动开发模式（前端 http://localhost:1420 + 桌面窗口）");
console.log("  修改前端代码热更新，关闭窗口或在终端按 Ctrl+C 退出\n");

const code = run(npmCmd, ["run", "tauri", "dev"], {
  cwd: ROOT,
  env: envWithCargo(pathDir),
});

if (code !== 0) {
  console.error(
    [
      "",
      "✖ 开发模式退出（非 0 退出码）。常见原因：",
      isWin
        ? "  · 未安装 MSVC 构建工具（Visual Studio C++ Build Tools）"
        : "  · 端口 1420 被占用（vite.config.ts 里是 strictPort）",
      "  · Rust 依赖首次编译较慢，如中途卡住可重试",
    ].join("\n"),
  );
}

process.exit(code);

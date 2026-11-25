// 开发与打包脚本共用的环境探测工具（macOS / Windows 通用）
import { spawnSync } from "node:child_process";
import { existsSync, readdirSync, readFileSync } from "node:fs";
import { homedir } from "node:os";
import { dirname, delimiter, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

export const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "..");
export const isWin = process.platform === "win32";
export const npmCmd = isWin ? "npm.cmd" : "npm";
export const exe = isWin ? ".exe" : "";

export function die(msg) {
  console.error(`\n✖ ${msg}\n`);
  process.exit(1);
}

function works(cmd) {
  const r = spawnSync(cmd, ["--version"], { stdio: "ignore", shell: isWin });
  return r.status === 0;
}

/** 同步执行命令，失败即抛出，返回退出码 */
export function run(cmd, args, opts = {}) {
  const r = spawnSync(cmd, args, { stdio: "inherit", shell: isWin, ...opts });
  if (r.error) die(`无法执行 ${cmd}：${r.error.message}`);
  return r.status ?? 1;
}

/**
 * 找到 cargo：优先 PATH，其次 ~/.cargo/bin，
 * 最后扫 ~/.rustup/toolchains（rustup 装了但没写 shim 的情况）。
 * 返回 { cargo, pathDir }，pathDir 需要拼进子进程 PATH。
 */
export function findCargo() {
  if (works("cargo")) return { cargo: "cargo", pathDir: null };

  const candidates = [
    join(homedir(), ".cargo", "bin", `cargo${exe}`),
  ];
  const toolchains = join(homedir(), ".rustup", "toolchains");
  if (existsSync(toolchains)) {
    for (const tc of readdirSync(toolchains)) {
      candidates.push(join(toolchains, tc, "bin", `cargo${exe}`));
    }
  }

  const found = candidates.find((p) => existsSync(p));
  return found ? { cargo: found, pathDir: dirname(found) } : { cargo: null, pathDir: null };
}

/** 生成带 cargo 所在目录的 env，避免 rustup 未进 PATH 时构建失败 */
export function envWithCargo(pathDir) {
  if (!pathDir) return process.env;
  const pathKey = isWin ? "Path" : "PATH";
  return { ...process.env, [pathKey]: `${pathDir}${delimiter}${process.env[pathKey] ?? ""}` };
}

export function ensureCargo() {
  const { cargo, pathDir } = findCargo();
  if (!cargo) {
    die(
      [
        "未找到 Rust / cargo，无法构建 Tauri 后端。",
        "",
        "  安装方式（任选其一）：",
        "    macOS / Linux:  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh",
        "    Windows:        下载并运行 https://rustup.rs 的安装包",
        "",
        "  装完重开终端再执行本脚本。",
      ].join("\n"),
    );
  }
  return { cargo, pathDir };
}

export function checkNodeVersion() {
  const major = Number(process.versions.node.split(".")[0]);
  if (major < 18) die(`Node.js 版本过低（当前 ${process.versions.node}），需要 18 及以上。`);
}

/** node_modules 缺失或不完整时安装依赖 */
export function ensureDeps() {
  const marker = join(ROOT, "node_modules", "@tauri-apps", "cli");
  if (existsSync(marker)) return;
  console.log("→ 首次运行，正在安装 npm 依赖…\n");
  const code = run(npmCmd, ["install"], { cwd: ROOT });
  if (code !== 0) die("npm install 失败，请检查网络或代理设置。");
}

export function readConf() {
  return JSON.parse(readFileSync(join(ROOT, "src-tauri", "tauri.conf.json"), "utf8"));
}

export function fmtSize(bytes) {
  return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
}

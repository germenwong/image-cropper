#!/usr/bin/env node
// 打包并归集产物：npm run pack [-- --debug] [-- --open]
// 产物输出到 release/<平台>/<版本>/，同时打印体积清单
import { spawnSync } from "node:child_process";
import { copyFileSync, existsSync, mkdirSync, readdirSync, rmSync, statSync } from "node:fs";
import { join } from "node:path";
import {
  ROOT,
  checkNodeVersion,
  die,
  ensureCargo,
  ensureDeps,
  envWithCargo,
  fmtSize,
  isWin,
  npmCmd,
  readConf,
  run,
} from "./env.mjs";

const args = new Set(process.argv.slice(2));
const debug = args.has("--debug");
const openAfter = args.has("--open");

checkNodeVersion();

const platformName = process.platform === "darwin" ? "macos" : process.platform === "win32" ? "windows" : null;
if (!platformName) {
  die(
    "当前只支持 macOS / Windows 打包。Tauri 不支持交叉编译安装包，" +
      "Windows 包需在 Windows 机器上跑 npm run pack。",
  );
}

const { pathDir } = ensureCargo();
ensureDeps();

const conf = readConf();
const { version, productName } = conf;
const profile = debug ? "debug" : "release";
const arch = process.arch === "arm64" ? "aarch64" : "x64";

console.log(`→ 打包 ${productName} v${version}（${platformName}/${arch}，${profile}）\n`);

const buildArgs = ["run", "tauri", "build"];
if (debug) buildArgs.push("--", "--debug");

const code = run(npmCmd, buildArgs, { cwd: ROOT, env: envWithCargo(pathDir) });
if (code !== 0) die("打包失败，请查看上方 Tauri / cargo 报错信息。");

// ---- 归集产物 ----
const targetDir = join(ROOT, "src-tauri", "target", profile);
const bundleDir = join(targetDir, "bundle");
const outDir = join(ROOT, "release", platformName, version);

rmSync(outDir, { recursive: true, force: true });
mkdirSync(outDir, { recursive: true });

const outputs = [];

function pickFiles(dir, ext) {
  if (!existsSync(dir)) return [];
  return readdirSync(dir)
    .filter((f) => f.toLowerCase().endsWith(ext))
    .map((f) => join(dir, f));
}

function copyFile(src, destName) {
  const dest = join(outDir, destName);
  copyFileSync(src, dest);
  outputs.push(dest);
}

if (platformName === "macos") {
  const app = pickFiles(join(bundleDir, "macos"), ".app")[0];
  if (app) {
    const dest = join(outDir, `${productName}_${version}.app`);
    const r = spawnSync("ditto", [app, dest], { stdio: "inherit" });
    if (r.status !== 0) die("复制 .app 失败（ditto 返回非 0）。");
    outputs.push(dest);
  }
  for (const dmg of pickFiles(join(bundleDir, "dmg"), ".dmg")) {
    copyFile(dmg, `${productName}_${version}_${arch}.dmg`);
  }
} else {
  for (const msi of pickFiles(join(bundleDir, "msi"), ".msi")) copyFile(msi, msi.split(/[\\/]/).pop());
  for (const setup of pickFiles(join(bundleDir, "nsis"), ".exe")) copyFile(setup, setup.split(/[\\/]/).pop());

  const portable = join(targetDir, "image-cropper.exe");
  if (existsSync(portable)) copyFile(portable, `${productName}_${version}_portable.exe`);
}

if (outputs.length === 0) {
  die(`未找到任何产物，请检查 ${bundleDir} 下是否有生成文件。`);
}

function sizeOf(p) {
  const st = statSync(p);
  if (!st.isDirectory()) return st.size;
  return readdirSync(p, { withFileTypes: true }).reduce((sum, e) => {
    const child = join(p, e.name);
    return sum + (e.isDirectory() ? sizeOf(child) : lstatSize(child));
  }, 0);
}

function lstatSize(p) {
  try {
    return statSync(p).size;
  } catch {
    return 0; // 断链的符号链接
  }
}

console.log("\n✔ 打包完成，产物已归集到：");
console.log(`  ${outDir}\n`);
for (const p of outputs) {
  console.log(`  ${fmtSize(sizeOf(p)).padStart(9)}  ${p.slice(outDir.length + 1)}`);
}

if (debug) console.log("\n提示：这是 debug 包，仅用于本地验证，不要分发。");
if (openAfter) {
  spawnSync(platformName === "macos" ? "open" : "explorer", [outDir], { stdio: "ignore" });
}

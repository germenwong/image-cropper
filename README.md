# 一键批量图片裁切工具（MVP V1.0）

设计师自用的本地批量居中裁切工具。选定素材文件夹 → 配置裁切参数 → 实时预览 → 一键批量导出，
替代在 Photoshop 中逐张手动裁切。纯本地运行，**不上传任何图片，无网络请求**。

技术栈：Tauri 2 + Vue 3 + TypeScript + Naive UI，图像处理使用 Rust `image` crate（无外部动态库依赖）。

---

## 功能

- 递归扫描素材目录，支持 `png / jpg / jpeg / webp`
- 两种居中裁切模式：
  - **规格输出**（默认）：目标尺寸既决定取景比例，也决定成品像素；宽图裁左右、高图裁上下
  - **像素裁切**：按目标像素从原图中心 1:1 截取，不缩放（适合精确抠图）
- 小图默认不放大，居中放置并按填充策略补边；规格输出下可勾选「允许放大原图」
- 填充策略：透明 / 纯色（含透明度） / 拉伸铺满
- 输出格式：PNG（无损，保留透明）/ JPG / WebP，可调质量（PNG 忽略）
- 左右双栏实时预览，画布自动适应窗口（小图放大、大图缩小），支持缩放、平移、双击复位；原图侧叠加显示裁切区域并标注保留的原图像素尺寸
- 输出保留原目录结构与文件名，重名自动追加 `_1`、`_2` 后缀，绝不覆盖原图
- 导出进度条 + 实时日志，失败项单独标注原因，损坏图片自动跳过不中断
- 支持拖拽文件夹到目录框、拖拽单张图片到预览区测试
- 尺寸预设可保存，点击一键填充
- 支持浅色 / 深色主题切换（右上角开关），选择本地记忆
- 一键清空目录、预览、日志与进度（不清除参数设置和尺寸预设，不动磁盘文件）

## 环境要求

| 用途 | 要求 |
| --- | --- |
| 前端构建 | Node.js 18+（推荐 20+） |
| Rust 后端 | Rust stable（rustup 安装） |
| macOS | Xcode Command Line Tools |
| Windows | MSVC 构建工具（Visual Studio C++ Build Tools） |

## 开发

```bash
npm run dev:app
```

一条命令搞定：检查 Node / Rust / Xcode CLT 环境 → 缺失依赖自动 `npm install` → 启动 Vite（1420）与 Tauri 桌面窗口，前端改动热更新。

- Rust 装在 rustup 里但 `cargo` 没进 PATH 时，脚本会自动去 `~/.cargo/bin`、`~/.rustup/toolchains/*/bin` 查找
- 只调前端样式可 `npm run dev` 在浏览器打开，但没有 Tauri 命令，图片相关功能不可用
- 手动等价命令：`npm install && npm run tauri dev`

## 打包

```bash
npm run pack          # release 包
npm run pack:debug    # debug 包，仅本地验证
```

打包脚本会读取 `src-tauri/tauri.conf.json` 的版本号，构建后把产物按平台归集到 `release/<平台>/<版本>/`，并打印体积清单。加 `-- --open` 可直接打开输出目录。

| 平台 | 产物 |
| --- | --- |
| macOS | `一键批量裁切_<版本>.app`、`一键批量裁切_<版本>_<arch>.dmg` |
| Windows | `..._x64_zh-CN.msi`、`..._x64-setup.exe`、`一键批量裁切_<版本>_portable.exe`（免安装） |

```bash
npm run pack -- --open          # 打包并打开产物目录
```

Tauri 不支持交叉编译安装包，**Windows 包必须在 Windows 机器上执行 `npm run pack`**（脚本是 Node 写的，macOS / Windows 共用一套，本机装有 Node 即可）。

手动等价命令：`npm run tauri build`，产物在 `src-tauri/target/release/bundle/`。

产品名是中文，WiX 默认代码页 1252 会导致 MSI 打包失败（LGHT0311），已在 `tauri.conf.json` 的 `bundle.windows.wix.language` 指定 `zh-CN`，安装包文件名因此带 `_zh-CN`。

## 测试

```bash
cd src-tauri
cargo test                                      # 单元测试 + 端到端导出测试
cargo run --example gen_testdata                # 生成测试素材到 src-tauri/testdata/src
cargo run --example run_batch_check             # 批量导出验证，结果输出到 src-tauri/testdata/out
```

## 目录结构

```
scripts/
  dev.mjs                # 一键启动调试
  build.mjs              # 打包 + 产物归集
  env.mjs                # 环境探测（Node / cargo / 版本号）
src/                     # Vue 前端
  views/MainView.vue     # 主界面：目录区 / 参数区 / 双栏预览 / 导出面板
  components/
    DirPicker.vue        # 目录选择 + 拖拽
    ParamPanel.vue       # 参数配置 + 尺寸预设
    PreviewPane.vue      # 预览画布（缩放/平移/裁切框）
  api.ts                 # Tauri command 与事件的封装
src-tauri/src/
  crop.rs                # 核心裁切算法 + 编码（含单测）
  scan.rs / scan_dir     # 目录递归扫描
  preview.rs             # 预览图渲染（限制最长边，避免大图卡顿）
  export.rs              # 批量导出（rayon 并发、重名处理、进度事件）
  presets.rs             # 尺寸预设持久化
  lib.rs                 # command 注册与参数校验
```

## 内存与并发

- 逐张处理，不一次性加载全部图片
- 并发线程数限制为 `min(CPU 核数, 8)`，避免同时解码多张超大图导致内存峰值
- 预览图统一降采样到最长边 1200px 再送前端

## 已知限制

- MVP 仅支持居中裁切，对齐方式后续版本扩展
- WebP 目前使用无损编码
- 未做代码签名与公证，首次打开 macOS 需右键 →「打开」
- 未做代码签名，Windows 首次运行安装包或 exe 会触发 SmartScreen「未知发布者」，需点「更多信息 → 仍要运行」

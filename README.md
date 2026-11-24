# 一键批量图片裁切工具（MVP V1.0）

设计师自用的本地批量居中裁切工具。选定素材文件夹 → 配置裁切参数 → 实时预览 → 一键批量导出，
替代在 Photoshop 中逐张手动裁切。纯本地运行，**不上传任何图片，无网络请求**。

技术栈：Tauri 2 + Vue 3 + TypeScript + Naive UI，图像处理使用 Rust `image` crate（无外部动态库依赖）。

---

## 功能

- 递归扫描素材目录，支持 `png / jpg / jpeg / webp`
- 统一居中裁切到指定像素尺寸；宽图裁左右、高图裁上下、小图不放大居中填充
- 填充策略：透明 / 纯色（含透明度） / 拉伸铺满
- 输出格式：PNG（无损，保留透明）/ JPG / WebP，可调质量（PNG 忽略）
- 左右双栏实时预览，画布支持缩放、平移、双击复位，原图侧叠加显示裁切区域
- 输出保留原目录结构与文件名，重名自动追加 `_1`、`_2` 后缀，绝不覆盖原图
- 导出进度条 + 实时日志，失败项单独标注原因，损坏图片自动跳过不中断
- 支持拖拽文件夹到目录框、拖拽单张图片到预览区测试
- 尺寸预设可保存，点击一键填充

## 环境要求

| 用途 | 要求 |
| --- | --- |
| 前端构建 | Node.js 18+（推荐 20+） |
| Rust 后端 | Rust stable（rustup 安装） |
| macOS | Xcode Command Line Tools |
| Windows | MSVC 构建工具（Visual Studio C++ Build Tools） |

## 开发

```bash
npm install
npm run tauri dev
```

## 打包

```bash
npm run tauri build
```

- macOS 产物：`src-tauri/target/release/bundle/macos/一键批量裁切.app` 与 `.dmg`
- Windows 产物：`bundle/msi/一键批量裁切_<版本>_x64_zh-CN.msi`、`bundle/nsis/一键批量裁切_<版本>_x64-setup.exe`，以及免安装单文件 `target/release/image-cropper.exe`

  产品名是中文，WiX 默认代码页 1252 会导致 MSI 打包失败（LGHT0311），已在 `tauri.conf.json` 的 `bundle.windows.wix.language` 指定 `zh-CN`，安装包文件名因此带 `_zh-CN`。

Windows 需在 Windows 机器上执行打包（Tauri 不支持跨平台交叉编译安装包）。

## 测试

```bash
cd src-tauri
cargo test                                      # 单元测试 + 端到端导出测试
cargo run --example gen_testdata                # 生成测试素材到 src-tauri/testdata/src
cargo run --example run_batch_check             # 批量导出验证，结果输出到 src-tauri/testdata/out
```

## 目录结构

```
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

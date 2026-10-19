<div align="center">

# 📸 PhotoSelect Desktop

### 极速摄影选片与管理平台

**Zero-Import, Millisecond RAW+JPG Culling Platform for Photographers**

专为商业摄影师、独立摄影师与摄影爱好者打造的高性能桌面暗房快速初筛工具。

[![Version](https://img.shields.io/badge/version-1.0.0-blue?style=flat-square)](https://github.com/chensiqi2007/photoselect)
[![Tauri](https://img.shields.io/badge/Tauri-2.0-FFC131?style=flat-square&logo=tauri&logoColor=white)](https://v2.tauri.app/)
[![Vue](https://img.shields.io/badge/Vue-3.5-4FC08D?style=flat-square&logo=vuedotjs&logoColor=white)](https://vuejs.org/)
[![Rust](https://img.shields.io/badge/Rust-Stable-DEA584?style=flat-square&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![TypeScript](https://img.shields.io/badge/TypeScript-5.6-3178C6?style=flat-square&logo=typescript&logoColor=white)](https://www.typescriptlang.org/)
[![Platform](https://img.shields.io/badge/platform-Windows%20|%20Linux-lightgrey?style=flat-square)](#)

<br/>

<a href="https://get.microsoft.com/installer/download/9mwnhgc08xkm?referrer=appbadge" target="_self">
	<img src="https://get.microsoft.com/images/en-us%20dark.svg" width="200" alt="Get it from Microsoft Store"/>
</a>

<br/>

</div>

---

## 🌟 核心特性

### ⚡ 零导入 · 即开即看
无需像 Lightroom Classic 那样漫长等待导入建库，直接选定磁盘或 SD 卡目录，毫秒级读取呈现。

### 🔗 RAW + JPG 智能同名绑定
自动将同名 JPG/PNG 与各大相机厂商 RAW 格式（CR2 / CR3 / ARW / NEF / RAF / DNG 等）聚合为单张卡片，杜绝重复筛选。

### 👁️ 智能人脸 / 人眼检视
内置离线 MediaPipe Face Landmarker 模型，自动检测画面人脸，截取双眼特写并计算合焦锐度评分与眨眼状态。

### 🎯 WebGL 2.0 硬件加速视口
- 原生 Shader 级 **峰值对焦 (Focus Peaking)**，一键显示高反差锐度边缘
- 18% 标准中性灰暗房视口，消除周围杂色对视觉曝光的干扰
- 100% 局部一键放大锁与平滑拖拽

### ⚖️ 双图联动对比模式
支持基准图 A 与候选图 B 并排比对，同步缩放与同步平移。

### ⌨️ 全键盘盲操暗房流

| 操作 | 快捷键 |
|------|--------|
| 评级打星 | `0` – `5` |
| 留用 / 淘汰 / 清除旗标 | `P` / `X` / `U` |
| 色彩标签 | `6` 红 · `7` 黄 · `8` 绿 · `9` 蓝 · `0` 清除 |
| 翻页导航 | `←` `→` 或 `A` `D` |
| 对比互换 | `S` |
| 峰值对焦 | `F` |

### 📦 快捷导出 & XMP 侧车
- 筛选后一键批量复制 / 移动留用 RAW 或 JPG
- 自动生成符合 Adobe Lightroom Classic 标准的 `.xmp` 附属文件，导入精修时评级标记无缝保留

### 🤖 AI 智能选片助手
支持配置 OpenAI 兼容格式视觉大模型，批量智能评估构图、表情与对焦并给出打星推荐。

### 🌐 多语言 & 主题
简体中文 · 繁体中文 · English — 深色暗房 & 浅色日间主题切换。

---

## 🛠️ 技术栈

| 层级 | 技术 |
|------|------|
| 桌面框架 | [Tauri 2.0](https://v2.tauri.app/) — Rust 跨平台底层 |
| 前端视图 | [Vue 3](https://vuejs.org/) Composition API + [TypeScript 5.6](https://www.typescriptlang.org/) |
| 状态管理 | [Pinia 2.2](https://pinia.vuejs.org/) |
| 构建工具 | [Vite 5.4](https://vitejs.dev/) + [TailwindCSS 3.4](https://tailwindcss.com/) |
| 离线 AI | `@mediapipe/tasks-vision` + WebAssembly 人脸模型 |
| 底层解码 | `rawler` · `image` · `rusqlite` · `tokio` · `rayon` |

---

## 🚀 快速上手

### 环境要求

| 依赖 | 最低版本 |
|------|----------|
| Node.js | ≥ 18.0 (推荐 20 / 22 LTS) |
| Rust | ≥ 1.78.0 (`rustup update stable`) |
| C++ 构建工具 | Windows: MSVC (Visual Studio C++ 生成工具) |

### 安装与运行

```bash
# 安装依赖
npm install

# 启动开发模式 (前端 + Tauri 联调)
npm run tauri:dev

# 仅调试前端
npm run dev
```

### 编译生产包

```bash
# 前端类型检查 + 打包
npm run build

# Windows 独立免安装 .exe
.\build-exe.bat        # 或 .\build-exe.ps1

# Microsoft Store MSIX 包
npm run build:msix

# Linux (DEB/RPM via WSL)
npm run build:linux
```

> 打包完成后，可执行文件输出至工程根目录及 `src-tauri/target/release/`。

---

## 📄 许可证

本项目遵循私有专有开发协议，未经许可严禁用于商业分发。

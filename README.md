# PhotoSelect Desktop 极速摄影选片与管理平台

> **Zero-Import, Millisecond RAW+JPG Culling Platform for Photographers**  
> 专为商业摄影师、独立摄影师与摄影爱好者打造的高性能桌面暗房快速初筛工具。

---

## 🌟 核心特性

- ⚡ **零导入与即开即看 (Zero-Import)**：无需像 Lightroom Classic 那样漫长等待导入建库，直接选定磁盘或 SD 卡目录，毫秒级读取呈现。
- 🔗 **RAW + JPG 智能同名绑定 (Stacking)**：自动将同名 JPG/PNG 与各大相机厂商 RAW 格式（CR2/CR3/ARW/NEF/RAF/DNG 等）聚合为单张卡片，杜绝重复筛选。
- 👁️ **智能人脸/人眼检视 (Eye AF Inspection)**：内置离线 MediaPipe Face Landmarker 模型，自动检测画面人脸、截取双眼特写并计算合焦锐度评分与眨眼状态。
- 🎯 **WebGL 2.0 硬件加速视口**：
  - 原生 Shader 级**峰值对焦 (Focus Peaking)**，一键显示高反差锐度边缘。
  - 18% 标准中性灰暗房视口，消除周围杂色对视觉曝光的干扰。
  - 100% 局部一键放大锁与平滑拖拽。
- ⚖️ **双图联动对比模式 (2-Up Compare)**：支持基准图 A 与候选图 B 并排比对，支持同步缩放与同步平移。
- ⌨️ **全键盘盲操暗房流 (Keyboard-First)**：
  - 评级打星：`0` - `5` 键
  - 旗标标记：`P` (留用/Pick) / `X` (淘汰/Reject) / `U` (清除旗标/Unflag)
  - 色彩标签：`6` (红) / `7` (黄) / `8` (绿) / `9` (蓝) / `0` (清除)
  - 翻页导航：`←` / `→` 或 `A` / `D`
  - 对比互换：`S` 键互换对比图位置
  - 峰值对焦：`F` 键开启/关闭
- 📦 **快捷导出与 XMP 侧车生成**：
  - 筛选后一键批量复制/移动留用 RAW 或 JPG。
  - 自动生成符合 Adobe Lightroom Classic 标准的 `.xmp` 附属文件，导入精修时评级标记无缝保留。
- 🤖 **AI 智能选片助手**：支持配置 OpenAI 兼容格式视觉大模型，批量智能评估构图、表情与对焦并给出打星推荐。
- 🌐 **多语言与外观主题**：支持简体中文、繁体中文、英文切换；支持深色暗房与浅色日间主题。

---

## 🛠️ 技术栈

- **桌面框架**：[Tauri 2.0](https://v2.tauri.app/) (Rust 跨平台底层)
- **前端视图**：[Vue 3](https://vuejs.org/) (Composition API, `<script setup>`) + [TypeScript 5.6](https://www.typescriptlang.org/)
- **状态管理**：[Pinia 2.2](https://pinia.vuejs.org/)
- **构建工具**：[Vite 5.4](https://vitejs.dev/) + [TailwindCSS 3.4](https://tailwindcss.com/)
- **离线 AI/视觉**：`@mediapipe/tasks-vision` + 本地 WebAssembly 运行时与人脸模型
- **底层解码与并发**：`rawler`、`image`、`rusqlite`、`tokio`、`rayon`

---

## 🚀 快速上手与本地开发

### 环境要求
1. **Node.js**：>= 18.0.0 (推荐 20 或 22 LTS)
2. **Rust**：>= 1.78.0 (`rustup update stable`)
3. **C++ 构建工具**：Windows 上安装 Visual Studio C++ 生成工具 (MSVC)

### 安装依赖
```bash
npm install
```

### 启动开发模式
```bash
npm run tauri:dev
```
> 或者分别调试前端：`npm run dev`

### 编译生产打包
```bash
# 前端类型检查与打包
npm run build

# 生成独立 Windows 免安装可执行文件 (.exe)
.\build-exe.bat
# 或使用 PowerShell
.\build-exe.ps1
```

打包完成后，应用程序可执行文件将输出至工程根目录及 `src-tauri/target/release/`。

---

## 📄 许可证

本项目遵循私有专有开发协议，未经许可严禁用于商业分发。

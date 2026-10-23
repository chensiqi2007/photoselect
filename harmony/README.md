# PhotoSelect HarmonyOS

鸿蒙 PC/2-in-1 独立子工程。ArkUI/ArkTS 提供原生界面，C++ NAPI 异步桥接 Rust；Rust 扫描核心复用桌面项目的 models.rs 和 scanner.rs 源码。所有鸿蒙适配文件集中在此目录，不改现有 Tauri 版本。

## 当前实现

- 扫描用户指定目录；Rust 支持递归开关、JPG/RAW/XMP 同名配对和自然排序。
- ArkUI 照片网格尝试显示标准图，提供 RAW+JPG 过滤、会话内星级和留用/淘汰标记。
- 尚未移植评级 SQLite 持久化、RAW 预览解码、相似度分析、完整导出。
- 页面提供 2-in-1 专用 DocumentViewPicker 目录选择入口；选择后由 CoreFileKit 在授权 URI 内枚举文件，再把文件 URI 清单传给 Rust 做配对，避免 Rust 绕过鸿蒙沙箱。仍需在目标设备验证外置卷、中文路径和系统版本差异。

## 构建

1. 使用支持 HarmonyOS 6.0.1 的 DevEco Studio 打开本目录。
2. 安装 Rust stable，并安装 `aarch64-unknown-linux-ohos` 与 `x86_64-unknown-linux-ohos` 标准库目标。
3. 在 PowerShell 执行 `scripts/build-rust.ps1`，脚本会自动使用 DevEco SDK 的 OpenHarmony NDK，为 ARM64 和 x86_64 交叉编译 Rust 静态库。可通过 `DEVECO_SDK_HOME` 覆盖默认 SDK 路径。
4. 使用 DevEco Sync Project 后构建 HAP；也可执行 `F:\DevEco Studio\tools\hvigor\bin\hvigorw.bat --mode module -p product=default assembleHap`。
5. 在鸿蒙 PC/2-in-1 设备验证目录授权、文件枚举、预览和窗口行为。

本地验证：Rust 核心的 3 项单元测试通过；aarch64 与 x86_64 OpenHarmony Rust 核心均已通过 DevEco NDK 交叉编译。HAP 构建成功并包含两个 ABI，产物位于 `entry/build/default/outputs/default/entry-default-unsigned.hap`，当前未签名。仍需签名后安装，并在设备或模拟器验证目录授权、文件枚举、预览和窗口行为。

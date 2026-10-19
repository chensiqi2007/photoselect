# PhotoSelect Desktop - 微软应用商店 (Microsoft Store) 上架指南

本文档指导如何将 **PhotoSelect Desktop** 应用程序打包为 **MSIX** 原生安装包或 **PWA**，并顺利提交至 **微软合作伙伴中心 (Microsoft Partner Center)** 进行上架发布。

---

## 目录
1. [技术选型：MSIX 与 PWA 对比](#1-技术选型msix-与-pwa-对比)
2. [方案一：MSIX 原生桌面应用打包与上架（推荐）](#2-方案一msix-原生桌面应用打包与上架推荐)
   - [2.1 目录结构与自动化工具](#21-目录结构与自动化工具)
   - [2.2 本地一键打包与安装测试](#22-本地一键打包与安装测试)
   - [2.3 微软合作伙伴中心 (Partner Center) 配置](#23-微软合作伙伴中心-partner-center-配置)
   - [2.4 生成 Store 正式包并提交审核](#24-生成-store-正式包并提交审核)
3. [方案二：PWA 渐进式 Web 应用配置与上架](#3-方案二pwa-渐进式-web-应用配置与上架)
   - [3.1 Web App Manifest 与 Service Worker](#31-web-app-manifest-与-service-worker)
   - [3.2 使用 PWABuilder 提交至微软应用商店](#32-使用-pwabuilder-提交至微软应用商店)
4. [常见问题与审核注意事项 (FAQ)](#4-常见问题与审核注意事项-faq)

---

## 1. 技术选型：MSIX 与 PWA 对比

| 比较维度 | MSIX 原生桌面应用（Desktop Bridge） | PWA 渐进式 Web 应用（Hosted Web App） |
| :--- | :--- | :--- |
| **底层引擎** | Tauri 2.0 (Rust 高性能多线程引擎) | 浏览器 Webview / Chromium 沙盒 |
| **相机 RAW 解码** | ✅ 原生支持 (`rawler` 极速解码 Sony/Canon/Nikon) | ❌ 受限（无原生 Rust 硬件解码，大图卡顿） |
| **本地文件访问** | ✅ 毫秒级多线程目录遍历与原图流式加载 | ❌ 浏览器安全沙箱限制，无法自由批量直读本地磁盘 |
| **离线与数据库** | ✅ 本地 SQLite 毫秒级并发缓存 | ⚠️ 仅 IndexedDB，容量与性能受限 |
| **商店体验** | 🌟 认证标志、现代沙盒隔离、干净卸载、自动更新 | 🌟 适用于在线服务型产品 |
| **结论** | **强烈推荐（摄影专业选片的核心方案）** | **适用于 Web 纯展示端或轻量版** |

> **建议**：PhotoSelect 作为一款主打 **“零导入、毫秒级 RAW+JPG 选片”** 的桌面级生产力工具，**必须优先采用 MSIX 方案** 上架微软应用商店，以保障核心性能！

---

## 2. 方案一：MSIX 原生桌面应用打包与上架（推荐）

本项目已内置完整的 MSIX 自动化构建流程，无需手动编写复杂的打包脚本。

### 2.1 目录结构与自动化工具

```
PhotoSelect_Desktop/
├── build-msix.ps1             # PowerShell 自动化打包主脚本 (支持编译、组装、测试签名)
├── build-msix.bat             # Windows 资源管理器一键双击批处理脚本
├── msix/
│   ├── msix-config.json       # 微软商店参数配置文件 (包名、Publisher ID、版本号)
│   ├── AppxManifest.xml       # Windows 10/11 Desktop Bridge 原生应用清单
│   ├── AppxManifest.xml.template # 清单模板 (自动注入配置参数)
│   └── Assets/                # 微软商店全部标准图标尺寸
│       ├── StoreLogo.png          # 50x50 (商店主图标)
│       ├── Square44x44Logo.png    # 44x44 (任务栏与列表图标)
│       ├── Square71x71Logo.png    # 71x71 (小磁贴)
│       ├── Square150x150Logo.png  # 150x150 (中磁贴)
│       ├── Square310x310Logo.png  # 310x310 (大磁贴)
│       ├── Wide310x150Logo.png    # 310x150 (宽磁贴)
│       └── SplashScreen.png       # 620x300 (启动闪屏)
└── release/                   # 打包生成目录 (*.msix)
```

### 2.2 本地一键打包与安装测试

在项目根目录下，直接运行以下命令之一：

```bash
# 完整打包（包含前端 npm run build 与 Rust cargo build）
npm run build:msix

# 快速打包（跳过编译，直接使用已有的 release 目录可执行文件，耗时仅 2 秒）
npm run build:msix:quick

# 或在 Windows 资源管理器中直接双击：
build-msix.bat
```

#### 本地测试安装步骤：
打包成功后，安装包保存在 `release/PhotoSelect-Desktop-v1.0.0.0.msix`。由于 Windows 要求安装包必须拥有受信任的数字签名，脚本已自动为您生成并签名了开发测试证书 `msix/PhotoSelect_Dev.pfx`。
1. **首次测试需信任证书**（以管理员身份运行终端）：
   ```powershell
   certutil -addstore -f Root "msix\PhotoSelect_Dev.pfx"
   ```
2. **安装应用**：
   - 直接双击 `release\PhotoSelect-Desktop-v1.0.0.0.msix` 点击安装；
   - 或在 PowerShell 中执行：
     ```powershell
     Add-AppxPackage -Path "release\PhotoSelect-Desktop-v1.0.0.0.msix"
     ```

---

### 2.3 微软合作伙伴中心 (Partner Center) 配置

1. 打开 [微软合作伙伴中心控制台](https://partner.microsoft.com/dashboard/) 并登录您的微软开发者账号。
2. 点击 **“创建新应用 (Create a new app)”** 并保留应用名称（例如：`PhotoSelect Desktop`）。
3. 进入应用的 **【产品管理 (Product management)】->【产品标识 (Product identity)】** 页面。
4. 复制以下三项关键标识：
   - **程序包名称 (Package/Identity/Name)**（例如：`12345YourStudio.PhotoSelectDesktop`）
   - **发布者 (Package/Identity/Publisher)**（例如：`CN=XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX`）
   - **发布者显示名称 (PublisherDisplayName)**（例如：`Your Studio Name`）
5. 打开项目中的 `msix/msix-config.json`，将上述真实信息替换填入：
   ```json
   {
     "PackageName": "12345YourStudio.PhotoSelectDesktop",
     "PackageDisplayName": "PhotoSelect Desktop",
     "PublisherId": "CN=XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX",
     "PublisherDisplayName": "Your Studio Name",
     "Version": "1.0.0.0",
     "Description": "极速摄影选片与管理桌面应用 - Zero-Import, Millisecond-level RAW+JPG Culling Platform",
     "BackgroundColor": "#121212",
     "Executable": "photoselect-desktop.exe"
   }
   ```

---

### 2.4 生成 Store 正式包并提交审核

更新好 `msix/msix-config.json` 后，执行以下命令生成商店专用安装包：

```bash
# 生成商店专用包 (跳过自签名，等待 Partner Center 官方权威签名)
npm run build:msix:store
```

生成的文件将位于：
`release\PhotoSelect-Desktop-v1.0.0.0.msix`

#### 在 Partner Center 中提交：
1. 进入您的应用提交页面（**Start your submission**）。
2. 在 **“程序包 (Packages)”** 步骤中，直接拖拽上传上述 `PhotoSelect-Desktop-v1.0.0.0.msix`。
3. 系统将自动校验并提取包名、版本号与支持的架构（x64）。
4. 在 **“应用描述 (Store listings)”** 中填写应用简介、上传软件截图（至少 1 张 1920x1080 暗房界面截图）。
5. 提交审核（通常在 24 ~ 48 小时内完成审核并全球上线）。

---

## 3. 方案二：PWA 渐进式 Web 应用配置与上架

如果希望将 PhotoSelect 作为 PWA 应用上架微软应用商店，项目已内置完整的 Web App Manifest 与 Service Worker 基础设施。

### 3.1 Web App Manifest 与 Service Worker

项目已配置以下标准文件：
- `public/manifest.webmanifest`：配置了 `name`, `short_name`, `theme_color (#0ea5e9)`, `background_color (#121212)`, `display: standalone` 以及从 44x44 到 512x512 的自适应高清图标。
- `public/sw.js`：提供离线缓存策略、现代 PWA 生命周期支持。
- `index.html`：已引入 manifest 关联标签与 Service Worker 安全注册代码。

### 3.2 使用 PWABuilder 提交至微软应用商店

1. 将前端编译产物部署至公开 HTTPS 域名（例如通过 Vercel、Cloudflare Pages 或自有服务器）。
2. 访问微软官方 PWA 转换工具：[PWABuilder](https://www.pwabuilder.com/)。
3. 输入您的网站 URL，点击 **Start** 检查 PWA 合规性。
4. 校验通过后，点击 **Package for Stores** -> 选择 **Windows (Microsoft Store)**。
5. 填入 Partner Center 的应用标识信息，PWABuilder 将直接生成用于上传的 `.zip` / `.msix` 包。
6. 上传至微软合作伙伴中心完成发布。

---

## 4. 常见问题与审核注意事项 (FAQ)

### Q1: MSIX 审核提示 `runFullTrust` 受限权限？
- **说明**：PhotoSelect 作为高性能桌面软件，需要通过 Rust 底层访问本地相册目录、USB 读卡器和多核硬件进行 RAW 解码，因此在 `AppxManifest.xml` 中声明了 `<rescap:Capability Name="runFullTrust" />`。
- **应对方案**：微软审核系统在初次提交时可能会询问“为什么需要 Full Trust 权限？”，只需在提交说明 (Submission notes) 中简要回复：
  > *"PhotoSelect is a professional photography culling application requiring direct high-speed local filesystem access and multi-threaded native hardware decoding for camera RAW files."*

### Q2: 后续版本更新流程？
- 递增 `package.json` 与 `msix/msix-config.json` 中的版本号（例如由 `1.0.0.0` 变更为 `1.0.1.0`）。
- 再次运行 `npm run build:msix:store`。
- 在 Partner Center 创建新提交，上传新版本的 `.msix` 即可，用户电脑上的微软应用商店会自动完成无缝静默增量更新。

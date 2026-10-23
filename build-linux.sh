#!/usr/bin/env bash
# PhotoSelect Desktop - Linux 打包向导 (.deb & .rpm)
set -e

# 确保使用的是 Linux 路径与环境
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"
export PATH="/usr/local/bin:/usr/bin:/bin:$HOME/.cargo/bin:$PATH"

echo "========================================================================"
echo "    PhotoSelect Desktop - Linux 原生安装包打包向导 (.deb & .rpm)        "
echo "========================================================================"
echo ""

# 1. 检查 Linux 原生编译工具链
echo "[1/4] 检查并安装 Linux 系统底层依赖..."

if command -v apt-get &> /dev/null; then
    sudo apt-get update -qq
    sudo apt-get install -y -qq \
        build-essential \
        curl \
        wget \
        file \
        pkg-config \
        libxdo-dev \
        libssl-dev \
        librsvg2-dev \
        libayatana-appindicator3-dev \
        libwebkit2gtk-4.1-dev \
        rpm
fi

# 检查 Linux 原生 Rust
if ! command -v cargo &> /dev/null; then
    echo "[*] 正在安装 Linux 原生 Rust 工具链..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
else
    source "$HOME/.cargo/env" 2>/dev/null || true
fi

# 检查 Linux 原生 Node.js
if ! command -v node &> /dev/null; then
    echo "[*] 正在安装 Linux 原生 Node.js 20 LTS..."
    curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
    sudo apt-get install -y nodejs
fi

echo "  - Node 版本: $(node -v)"
echo "  - Cargo 版本: $(cargo -v | head -n 1)"

# 2. 编译前端
echo ""
echo "[2/4] 编译前端 WebGL 暗房界面 (npm run build)..."
npm install
npm run build

# 3. 编译 Tauri 后端并生成原生 .deb 与 .rpm
echo ""
echo "[3/4] 调用 Linux 原生 Tauri 编译 Rust 底座并打包..."
# 让 Tauri 分别生成 Debian 和 RPM 格式，保留各自正确的包元数据。
npx tauri build --bundles deb,rpm

# 4. 生成 .rpm 包
echo ""
echo "[4/4] 收集原生 RedHat / Fedora / openSUSE 安装包..."
mkdir -p release

# 只收集当前版本的原生包，避免复用 target 目录里的旧版 RPM。
APP_VERSION=$(node -p "JSON.parse(require('fs').readFileSync('src-tauri/tauri.conf.json', 'utf8')).version")
DEB_FILE=$(find src-tauri/target/release/bundle/deb -type f -name "*${APP_VERSION}*.deb" -print -quit)
RPM_FILE=$(find src-tauri/target/release/bundle/rpm -type f -name "*${APP_VERSION}*.rpm" -print -quit)

if [ -f "$DEB_FILE" ]; then
    echo "找到 DEB 包: $DEB_FILE"
    cp -f "$DEB_FILE" release/
    
else
    echo "[!] 未找到版本 $APP_VERSION 的 deb 包，请查看编译日志" >&2
    exit 1
fi

if [ -f "$RPM_FILE" ]; then
    echo "找到原生 RPM 包: $RPM_FILE"
    cp -f "$RPM_FILE" release/
else
    echo "[!] 未找到版本 $APP_VERSION 的原生 RPM 包，请查看编译日志" >&2
    exit 1
fi

echo ""
echo "========================================================================"
echo " [√] Linux 安装包打包完成！生成文件位于 release/ 目录下："
echo "========================================================================"
ls -lh release/*.deb release/*.rpm 2>/dev/null || true
echo "========================================================================"

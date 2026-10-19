#!/usr/bin/env bash
# PhotoSelect Desktop - Linux 打包向导 (.deb & .rpm)
set -e

# 确保使用的是 Linux 路径与环境
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
        rpm \
        alien
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

# 3. 编译 Tauri 后端并生成 .deb
echo ""
echo "[3/4] 调用 Linux 原生 Tauri 编译 Rust 底座并打包..."
# 在 Linux 环境下直接运行 tauri build 即可根据 targets: all 自动输出 deb 与 appimage
npx tauri build

# 4. 生成 .rpm 包
echo ""
echo "[4/4] 转换并生成 RedHat / Fedora / openSUSE .rpm 安装包..."
mkdir -p release

# 查找生成的 deb 包
DEB_FILE=$(find src-tauri/target/release/bundle/deb -name "*.deb" 2>/dev/null | head -n 1)

if [ -f "$DEB_FILE" ]; then
    echo "找到 DEB 包: $DEB_FILE"
    cp "$DEB_FILE" release/
    
    if command -v alien &> /dev/null; then
        echo "正在使用 alien 将 .deb 转换为标准 .rpm..."
        sudo alien -r -c --target=x86_64 "$DEB_FILE"
        RPM_GEN=$(find . -maxdepth 1 -name "*.rpm" 2>/dev/null | head -n 1)
        if [ -f "$RPM_GEN" ]; then
            mv "$RPM_GEN" release/
            echo "[√] RPM 生成完毕！"
        fi
    fi
else
    echo "[!] 未在 src-tauri/target/release/bundle/deb 找到 deb 包，请查看编译日志"
fi

echo ""
echo "========================================================================"
echo " [√] Linux 安装包打包完成！生成文件位于 release/ 目录下："
echo "========================================================================"
ls -lh release/*.deb release/*.rpm 2>/dev/null || true
echo "========================================================================"

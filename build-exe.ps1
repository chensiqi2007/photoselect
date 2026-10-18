# PhotoSelect Desktop - Windows App (EXE) 打包脚本 (PowerShell)
$ErrorActionPreference = "Stop"
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8

Write-Host "========================================================================" -ForegroundColor Cyan
Write-Host "       PhotoSelect Desktop - 极速摄影选片与管理平台 Windows 打包向导       " -ForegroundColor Green
Write-Host "========================================================================" -ForegroundColor Cyan
Write-Host ""

# 1. 检查 Rust 环境
$cargoPath = Get-Command cargo -ErrorAction SilentlyContinue
if (-not $cargoPath) {
    $userCargo = "$env:USERPROFILE\.cargo\bin"
    if (Test-Path "$userCargo\cargo.exe") {
        $env:PATH = "$userCargo;$env:PATH"
        $cargoPath = Get-Command cargo -ErrorAction SilentlyContinue
    }
}

if (-not $cargoPath) {
    Write-Host "[!] 未在当前系统 PATH 中检测到 Rust 编译工具链 (cargo / rustc)" -ForegroundColor Yellow
    Write-Host "[*] 本项目基于 Tauri 2.0 (Rust 高性能底层 + WebGL 2.0 前端) 构建 Windows .exe" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "正在尝试自动下载安装 Rust (rustup-init.exe)..." -ForegroundColor White
    $installerPath = "$env:TEMP\rustup-init.exe"
    try {
        [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
        Invoke-WebRequest -Uri "https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe" -OutFile $installerPath
        Write-Host "[*] 启动无人值守安装 (默认 MSVC 工具链)..." -ForegroundColor Green
        Start-Process -FilePath $installerPath -ArgumentList "-y" -Wait
        $env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"
    } catch {
        Write-Host "[X] 自动下载失败，请手动访问官方网站安装 Rust: https://rustup.rs/" -ForegroundColor Red
        Write-Host "安装完成后重新运行本脚本即可一键完成封装！" -ForegroundColor Yellow
        Read-Host "按回车键退出..."
        exit 1
    }
}

Write-Host "[1/3] 检查环境工具链..." -ForegroundColor Green
cargo --version
node -v

Write-Host ""
Write-Host "[2/3] 编译前端暗房界面 (Vue 3 + WebGL 2.0 + Vite)..." -ForegroundColor Green
npm run build

Write-Host ""
Write-Host "[3/3] 调用 Tauri 2.0 编译 Rust 后端并封装为原生 Windows .exe / .msi 安装包..." -ForegroundColor Green
npx tauri build

if ($LASTEXITCODE -eq 0) {
    Write-Host ""
    Write-Host "========================================================================" -ForegroundColor Green
    Write-Host " [√] 打包成功！Windows .exe 应用程序已就绪！" -ForegroundColor Green
    Write-Host " 可执行文件: src-tauri\target\release\PhotoSelect Desktop.exe" -ForegroundColor Cyan
    Write-Host " 安装包目录: src-tauri\target\release\bundle\" -ForegroundColor Cyan
    Write-Host "========================================================================" -ForegroundColor Green
    
    $releaseDir = Join-Path $PSScriptRoot "src-tauri\target\release"
    if (Test-Path $releaseDir) {
        Invoke-Item $releaseDir
    }
} else {
    Write-Host "[X] 打包失败，请检查编译日志" -ForegroundColor Red
}

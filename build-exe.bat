@echo off
chcp 65001 >nul
title PhotoSelect Desktop - Windows App (EXE) 自动化打包器
echo ========================================================================
echo        PhotoSelect Desktop - 极速摄影选片与管理平台 
echo                   Windows 原生 .EXE 封装打包程序
echo ========================================================================
echo.

REM 1. 检查环境变量或默认目录中是否存在 Cargo/Rust
where cargo >nul 2>nul
if %errorlevel% neq 0 (
    if exist "%USERPROFILE%\.cargo\bin\cargo.exe" (
        set "PATH=%USERPROFILE%\.cargo\bin;%PATH%"
    )
)

where cargo >nul 2>nul
if %errorlevel% neq 0 (
    echo [!] 未检测到 Rust 编译工具链 (cargo / rustc)
    echo [*] Tauri 2.0 采用 Rust 编写高性能底层引擎，需要 Rust 编译环境。
    echo.
    echo 正在为您自动下载官方 Rust 安装器 (rustup-init.exe)...
    powershell -Command "[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12; Invoke-WebRequest -Uri 'https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe' -OutFile '%TEMP%\rustup-init.exe'"
    if exist "%TEMP%\rustup-init.exe" (
        echo [*] 正在启动 Rust 安装向导，请在弹出窗口中按 1 并回车使用默认选项安装...
        "%TEMP%\rustup-init.exe" -y
        set "PATH=%USERPROFILE%\.cargo\bin;%PATH%"
    ) else (
        echo [X] 自动下载失败，请手动访问 https://rustup.rs/ 下载并安装 rustup。
        pause
        exit /b 1
    )
)

echo [1/3] 检查构建环境...
cargo --version
node -v

echo.
echo [2/3] 正在编译前端暗房界面 (Vue 3 + WebGL + Vite)...
call npm run build
if %errorlevel% neq 0 (
    echo [X] 前端编译失败，请检查报错信息。
    pause
    exit /b %errorlevel%
)

echo.
echo [3/3] 正在调用 Tauri 2.0 编译 Rust 后端并封装为 Windows .EXE 安装包 (NSIS Setup)...
call npx tauri build --bundles nsis

if %errorlevel% equ 0 (
    copy /y "src-tauri\target\release\photoselect-desktop.exe" "PhotoSelect-Desktop-v1.0.0.exe" >nul
    copy /y "src-tauri\target\release\bundle\nsis\PhotoSelect Desktop_1.0.0_x64-setup.exe" "PhotoSelect-Desktop-v1.0.0-Setup.exe" >nul
    echo.
    echo ========================================================================
    echo  [√] 打包成功！
    echo  1. Windows 安装包 (向导): PhotoSelect-Desktop-v1.0.0-Setup.exe
    echo  2. 独立免安装运行文件:     PhotoSelect-Desktop-v1.0.0.exe
    echo  3. NSIS 构建目录:         src-tauri\target\release\bundle\nsis\
    echo ========================================================================
    explorer.exe /select,"PhotoSelect-Desktop-v1.0.0-Setup.exe"
) else (
    echo.
    echo [X] 打包过程出现异常，请查看上方详细日志。
)

pause

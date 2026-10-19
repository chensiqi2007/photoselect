@echo off
chcp 65001 >nul
title PhotoSelect Desktop - 微软应用商店 (MSIX) 一键打包工具
echo ========================================================================
echo        PhotoSelect Desktop - 极速摄影选片与管理平台 
echo                   微软应用商店 MSIX 一键打包器
echo ========================================================================
echo.

powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0build-msix.ps1" %*

if %errorlevel% neq 0 (
    echo.
    echo [X] 打包遇到异常，请检查上方日志。
)

echo.
pause

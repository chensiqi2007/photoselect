# PhotoSelect Desktop - Linux (.deb and .rpm) Packager via WSL
$ErrorActionPreference = "Stop"

Write-Host "========================================================================" -ForegroundColor Cyan
Write-Host "    PhotoSelect Desktop - Linux Package Builder (.deb and .rpm)          " -ForegroundColor Green
Write-Host "========================================================================" -ForegroundColor Cyan
Write-Host ""

$wslCmd = Get-Command wsl.exe -ErrorAction SilentlyContinue
if (-not $wslCmd) {
    Write-Host "ERROR: WSL not detected. Install with: wsl --install -d Ubuntu" -ForegroundColor Red
    exit 1
}

Write-Host "Starting WSL Ubuntu to build Linux packages..." -ForegroundColor Green
Write-Host ""

$scriptDir = $PSScriptRoot
$wslPath = & wsl.exe -d Ubuntu -e wslpath $scriptDir

$bashScript = "cd '$wslPath'; sed -i 's/\r$//' ./build-linux.sh; chmod +x ./build-linux.sh; ./build-linux.sh"

& wsl.exe -d Ubuntu -e bash -c $bashScript

if ($LASTEXITCODE -eq 0) {
    Write-Host ""
    Write-Host "========================================================================" -ForegroundColor Green
    Write-Host "  Linux packages built successfully! Output: release/                    " -ForegroundColor Green
    Write-Host "========================================================================" -ForegroundColor Green
    $releasePath = Join-Path $PSScriptRoot "release"
    if (Test-Path $releasePath) {
        Invoke-Item $releasePath
    }
} else {
    Write-Host ""
    Write-Host "ERROR: Build failed. Check the logs above." -ForegroundColor Red
}

<#
.SYNOPSIS
    PhotoSelect Desktop - Windows MSIX Packaging Script for Microsoft Store
.DESCRIPTION
    Packages Tauri 2.0 native Win32 app into a Windows MSIX container.
    Supports both local testing (self-signed cert) and Microsoft Store submission.
#>

param(
    [switch]$SkipBuild,
    [switch]$StoreReady
)

$ErrorActionPreference = "Stop"
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8

Write-Host "========================================================================" -ForegroundColor Cyan
Write-Host "         PhotoSelect Desktop - Microsoft Store MSIX Build Tool          " -ForegroundColor Green
Write-Host "========================================================================" -ForegroundColor Cyan
Write-Host ""

$projectRoot = $PSScriptRoot
$msixDir = Join-Path $projectRoot "msix"
$configFile = Join-Path $msixDir "msix-config.json"
$templateFile = Join-Path $msixDir "AppxManifest.xml.template"
$releaseDir = Join-Path $projectRoot "release"
$targetExe = Join-Path $projectRoot "src-tauri\target\release\photoselect-desktop.exe"

# 1. Read configuration
if (-not (Test-Path $configFile)) {
    Write-Host "[X] Configuration file not found: $configFile" -ForegroundColor Red
    exit 1
}

$config = Get-Content $configFile -Raw -Encoding UTF8 | ConvertFrom-Json
$pkgName = $config.PackageName
$pkgVersion = $config.Version
$publisherId = $config.PublisherId

Write-Host "[1/5] Load MSIX Configuration:" -ForegroundColor Yellow
Write-Host "  - PackageName:          $pkgName"
Write-Host "  - PackageDisplayName:   $($config.PackageDisplayName)"
Write-Host "  - Version:              $pkgVersion"
Write-Host "  - Publisher:            $publisherId"
Write-Host ""

# 2. Locate Windows SDK Tools (makeappx.exe & signtool.exe)
Write-Host "[2/5] Locating Windows SDK tools..." -ForegroundColor Green
$sdkBins = Get-ChildItem "C:\Program Files (x86)\Windows Kits\10\bin" -ErrorAction SilentlyContinue |
           Where-Object { $_.PSIsContainer -and $_.Name -match '^\d+\.' } |
           Sort-Object Name -Descending

$makeappxPath = $null
$signtoolPath = $null

foreach ($sdk in $sdkBins) {
    $mTest = Join-Path $sdk.FullName "x64\makeappx.exe"
    $sTest = Join-Path $sdk.FullName "x64\signtool.exe"
    if ((Test-Path $mTest) -and (Test-Path $sTest)) {
        $makeappxPath = $mTest
        $signtoolPath = $sTest
        Write-Host "  [OK] Found Windows SDK: $($sdk.Name)" -ForegroundColor Cyan
        break
    }
}

if (-not $makeappxPath) {
    $cmdMake = Get-Command makeappx.exe -ErrorAction SilentlyContinue
    $cmdSign = Get-Command signtool.exe -ErrorAction SilentlyContinue
    if ($cmdMake -and $cmdSign) {
        $makeappxPath = $cmdMake.Source
        $signtoolPath = $cmdSign.Source
    } else {
        Write-Host "[X] makeappx.exe / signtool.exe not found. Please install Windows 10/11 SDK." -ForegroundColor Red
        exit 1
    }
}

# 3. Build application if not skipped
if (-not $SkipBuild) {
    Write-Host ""
    Write-Host "[3/5] Compiling PhotoSelect Desktop standalone release binary..." -ForegroundColor Green
    Write-Host "  -> Invoking Tauri CLI (npx tauri build --no-bundle) with embedded assets..." -ForegroundColor Gray
    
    npx tauri build --no-bundle
    if ($LASTEXITCODE -ne 0) {
        Write-Host "[X] Tauri release build failed!" -ForegroundColor Red
        exit 1
    }
} else {
    Write-Host ""
    Write-Host "[3/5] Skipping compilation, using existing binary..." -ForegroundColor Yellow
}

if (-not (Test-Path $targetExe)) {
    Write-Host "[X] Binary target not found: $targetExe" -ForegroundColor Red
    exit 1
}

# 4. Prepare Staging Directory
Write-Host ""
Write-Host "[4/5] Preparing MSIX staging directory and AppxManifest.xml..." -ForegroundColor Green
$stagingDir = Join-Path $msixDir "staging"
if (Test-Path $stagingDir) {
    Remove-Item $stagingDir -Recurse -Force
}
New-Item -ItemType Directory -Path "$stagingDir\Assets" -Force | Out-Null

Copy-Item $targetExe "$stagingDir\photoselect-desktop.exe" -Force

$assetsSrc = Join-Path $msixDir "Assets"
Copy-Item "$assetsSrc\*" "$stagingDir\Assets\" -Force

$manifestTemplate = Get-Content $templateFile -Raw -Encoding UTF8
$manifestContent = $manifestTemplate
$manifestContent = $manifestContent.Replace("{{PackageName}}", [string]$config.PackageName)
$manifestContent = $manifestContent.Replace("{{PackageDisplayName}}", [string]$config.PackageDisplayName)
$manifestContent = $manifestContent.Replace("{{PublisherId}}", [string]$config.PublisherId)
$manifestContent = $manifestContent.Replace("{{PublisherDisplayName}}", [string]$config.PublisherDisplayName)
$manifestContent = $manifestContent.Replace("{{Version}}", [string]$config.Version)
$manifestContent = $manifestContent.Replace("{{Description}}", [string]$config.Description)
$manifestContent = $manifestContent.Replace("{{BackgroundColor}}", [string]$config.BackgroundColor)
$manifestContent = $manifestContent.Replace("{{Executable}}", [string]$config.Executable)

$manifestOut = Join-Path $stagingDir "AppxManifest.xml"
[System.IO.File]::WriteAllText($manifestOut, $manifestContent, [System.Text.Encoding]::UTF8)

if (-not (Test-Path $releaseDir)) {
    New-Item -ItemType Directory -Path $releaseDir -Force | Out-Null
}

$outputMsix = Join-Path $releaseDir "PhotoSelect-Desktop-v$pkgVersion.msix"

Write-Host "  -> Running makeappx pack..." -ForegroundColor Gray
& $makeappxPath pack /d $stagingDir /p $outputMsix /o
if ($LASTEXITCODE -ne 0) {
    Write-Host "[X] makeappx pack failed!" -ForegroundColor Red
    exit 1
}

Write-Host "  [OK] Generated MSIX package: $outputMsix" -ForegroundColor Cyan

# 5. Signing logic
Write-Host ""
if ($StoreReady) {
    Write-Host "[5/5] StoreReady Mode: Kept unsigned (Microsoft Partner Center will sign it upon ingestion)." -ForegroundColor Yellow
} else {
    Write-Host "[5/5] Local test signing (Self-signed certificate)..." -ForegroundColor Green
    
    $cert = Get-ChildItem Cert:\CurrentUser\My | Where-Object { $_.Subject -eq $publisherId } | Select-Object -First 1
    if (-not $cert) {
        Write-Host "  -> Generating self-signed certificate for [$publisherId]..." -ForegroundColor Gray
        $cert = New-SelfSignedCertificate -Type Custom -Subject $publisherId `
            -KeyUsage DigitalSignature `
            -FriendlyName "PhotoSelect Dev Cert" `
            -CertStoreLocation "Cert:\CurrentUser\My" `
            -TextExtension @("2.5.29.37={text}1.3.6.1.5.5.7.3.3")
    }

    $certPfx = Join-Path $msixDir "PhotoSelect_Dev.pfx"
    $certPassword = "PhotoSelectDevPassword123!"
    $securePassword = ConvertTo-SecureString -String $certPassword -Force -AsPlainText
    Export-PfxCertificate -Cert $cert -FilePath $certPfx -Password $securePassword | Out-Null

    Write-Host "  -> Signing MSIX with signtool..." -ForegroundColor Gray
    & $signtoolPath sign /fd SHA256 /a /f $certPfx /p $certPassword $outputMsix
    if ($LASTEXITCODE -eq 0) {
        Write-Host "  [OK] Local test signature succeeded!" -ForegroundColor Green
    } else {
        Write-Host "  [!] Signature failed, but unsigned package is still intact." -ForegroundColor Yellow
    }
}

Remove-Item $stagingDir -Recurse -Force -ErrorAction SilentlyContinue

Write-Host ""
Write-Host "========================================================================" -ForegroundColor Green
Write-Host "                 [OK] MSIX Package Built Successfully!                  " -ForegroundColor Green
Write-Host "========================================================================" -ForegroundColor Green
Write-Host "Package Output: $outputMsix" -ForegroundColor Cyan
Write-Host ""
Write-Host "[Local Installation & Testing]:" -ForegroundColor White
Write-Host " 1. For first-time local installation, trust the dev certificate:" -ForegroundColor Gray
Write-Host "    certutil -addstore -f Root `"$msixDir\PhotoSelect_Dev.pfx`"" -ForegroundColor Yellow
Write-Host " 2. Double-click $outputMsix or run:" -ForegroundColor Gray
Write-Host "    Add-AppxPackage -Path `"$outputMsix`"" -ForegroundColor Yellow
Write-Host ""
Write-Host "[Microsoft Partner Center Upload]:" -ForegroundColor White
Write-Host " 1. Login to https://partner.microsoft.com/dashboard" -ForegroundColor Gray
Write-Host " 2. Open Product Management -> Product Identity to find your real Identity & Publisher." -ForegroundColor Gray
Write-Host " 3. Update msix/msix-config.json and run:" -ForegroundColor Gray
Write-Host "    powershell -ExecutionPolicy Bypass -File .\build-msix.ps1 -SkipBuild -StoreReady" -ForegroundColor Yellow
Write-Host " 4. Upload the generated .msix file under the 'Packages' page in Partner Center!" -ForegroundColor Gray
Write-Host "========================================================================" -ForegroundColor Green

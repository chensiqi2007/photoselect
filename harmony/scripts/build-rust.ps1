$ErrorActionPreference = 'Stop'
$HarmonyRoot = Split-Path -Parent $PSScriptRoot
$Manifest = Join-Path $HarmonyRoot 'rust-core\Cargo.toml'
$Targets = @(
  @{ Triple = 'aarch64-unknown-linux-ohos'; Abi = 'arm64-v8a'; RustTarget = 'aarch64-unknown-linux-ohos'; LinkTarget = 'aarch64-linux-ohos' },
  @{ Triple = 'x86_64-unknown-linux-ohos'; Abi = 'x86_64'; RustTarget = 'x86_64-unknown-linux-ohos'; LinkTarget = 'x86_64-linux-ohos' }
)
$EnvironmentNames = @(
  'NODE_HOME', 'DEVECO_SDK_HOME', 'HOS_SDK_HOME', 'OHOS_BASE_SDK_HOME', 'PATH',
  'CARGO_TARGET_AARCH64_UNKNOWN_LINUX_OHOS_LINKER', 'CARGO_TARGET_X86_64_UNKNOWN_LINUX_OHOS_LINKER', 'CARGO_ENCODED_RUSTFLAGS'
)
$PreviousEnvironment = @{}
foreach ($Name in $EnvironmentNames) { $PreviousEnvironment[$Name] = [Environment]::GetEnvironmentVariable($Name, 'Process') }
try {
  $NodeHome = if ($env:NODE_HOME -and (Test-Path (Join-Path $env:NODE_HOME 'node.exe'))) { $env:NODE_HOME } else { 'C:\Program Files\nodejs' }
  $SdkHome = if ($env:DEVECO_SDK_HOME -and (Test-Path (Join-Path $env:DEVECO_SDK_HOME 'default\openharmony\native'))) { $env:DEVECO_SDK_HOME } else { 'F:\DevEco Studio\sdk' }
  $NativeSdk = Join-Path $SdkHome 'default\openharmony\native'
  $Sysroot = Join-Path $NativeSdk 'sysroot'
  $LlvmBin = Join-Path $NativeSdk 'llvm\bin'
  if (-not (Test-Path (Join-Path $LlvmBin 'clang.exe'))) { throw "OpenHarmony NDK clang not found: $LlvmBin" }
  if (-not (Test-Path (Join-Path $Sysroot 'usr'))) { throw "OpenHarmony NDK sysroot not found: $Sysroot" }
  $env:DEVECO_SDK_HOME = $SdkHome
  $env:HOS_SDK_HOME = $SdkHome
  $env:OHOS_BASE_SDK_HOME = $SdkHome
  $env:NODE_HOME = $NodeHome
  $env:PATH = "$NodeHome;$env:PATH"

  $InstalledTargets = @(rustup target list --installed)
  foreach ($Target in $Targets) {
    if ($InstalledTargets -notcontains $Target.RustTarget) {
      throw "Install Rust target first with rustup target add $($Target.RustTarget)"
    }
  }

  foreach ($Target in $Targets) {
    $env:CARGO_TARGET_AARCH64_UNKNOWN_LINUX_OHOS_LINKER = Join-Path $LlvmBin 'clang.exe'
    $env:CARGO_TARGET_X86_64_UNKNOWN_LINUX_OHOS_LINKER = Join-Path $LlvmBin 'clang.exe'
    $env:CARGO_ENCODED_RUSTFLAGS = @(
      '-C',
      "link-arg=--target=$($Target.LinkTarget)",
      '-C',
      "link-arg=--sysroot=$Sysroot"
    ) -join [char]31
    cargo build --release --target $Target.Triple --manifest-path $Manifest
    if ($LASTEXITCODE -ne 0) {
      throw "Rust cross-compilation failed for $($Target.Triple). Configure the target OpenHarmony linker and sysroot."
    }
    $BuiltLibrary = Join-Path $HarmonyRoot "rust-core\target\$($Target.Triple)\release\libphotoselect_harmony_core.a"
    if (-not (Test-Path -LiteralPath $BuiltLibrary)) { throw "Rust library not found: $BuiltLibrary" }
    $NativeLibraryDir = Join-Path $HarmonyRoot "entry\src\main\cpp\prebuilt\$($Target.Abi)"
    New-Item -ItemType Directory -Force -Path $NativeLibraryDir | Out-Null
    $NativeLibrary = Join-Path $NativeLibraryDir 'libphotoselect_harmony_core.a'
    Copy-Item -LiteralPath $BuiltLibrary -Destination $NativeLibrary -Force
    Write-Host "Rust core ready: $NativeLibrary"
  }
} finally {
  foreach ($Name in $EnvironmentNames) {
    $PreviousValue = $PreviousEnvironment[$Name]
    if ($null -eq $PreviousValue) {
      Remove-Item -Path "Env:$Name" -ErrorAction SilentlyContinue
    } else {
      Set-Item -Path "Env:$Name" -Value $PreviousValue
    }
  }
}

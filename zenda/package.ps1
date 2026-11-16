<#
.SYNOPSIS
  Build Zenda's PhotoCraft for Work and pack the portable zip Work downloads.
.DESCRIPTION
  Builds photocraft.exe (with the zenda and heif features) and photocraft-cli.exe, and writes
  dist\zenda\photocraft-<version>-z<n>-windows-x64-portable.zip. Inside, one folder named
  photocraft-<version>-windows-x64-portable holds the binaries, so Work's manifest `exe` is
  photocraft-<version>-windows-x64-portable/photocraft.exe. No portable.txt: Work points
  PHOTOCRAFT_CONFIG_DIR at the tool's profile, which takes precedence anyway.
  Prints the zip's size and SHA-256 for app/manifest.json in zenda-work.
.EXAMPLE
  pwsh zenda/package.ps1 -Z 1
#>
param([Parameter(Mandatory)] [int] $Z, [switch] $SkipBuild)
$ErrorActionPreference = 'Stop'
$Root = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$Version = $null
$inPkg = $false
foreach ($line in Get-Content (Join-Path $Root 'Cargo.toml')) {
  if ($line -match '^\s*\[') { $inPkg = ($line.Trim() -eq '[workspace.package]'); continue }
  if ($inPkg -and $line -match '^\s*version\s*=\s*"([^"]+)"') { $Version = $Matches[1]; break }
}
if (-not $Version) { throw 'could not read [workspace.package] version from Cargo.toml' }
if (-not $env:PHOTOCRAFT_BUILD_SHA) { $env:PHOTOCRAFT_BUILD_SHA = (git -C $Root rev-parse HEAD) }
if (-not $env:PHOTOCRAFT_BUILD_DATE) { $env:PHOTOCRAFT_BUILD_DATE = (Get-Date).ToUniversalTime().ToString('yyyy-MM-dd') }
$Target = 'x86_64-pc-windows-msvc'
if (-not $SkipBuild) {
  # Static CRT, as upstream's package.ps1: no Visual C++ redistributable needed.
  $env:CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_RUSTFLAGS = '-C target-feature=+crt-static'
  $env:PHOTOCRAFT_REQUIRE_WINRES = '1'
  cargo build --release --locked -p photocraft -p photocraft-cli --features 'photocraft/heif photocraft/zenda' --target $Target
  if ($LASTEXITCODE -ne 0) { throw "cargo build failed ($LASTEXITCODE)" }
}
$TargetDir = if ($env:CARGO_TARGET_DIR) { $env:CARGO_TARGET_DIR } else { Join-Path $Root 'target' }
$Bin = Join-Path $TargetDir "$Target\release"
$Dist = Join-Path $Root 'dist\zenda'
$Folder = "photocraft-$Version-windows-x64-portable"
$Stage = Join-Path $Dist $Folder
if (Test-Path $Stage) { throw "Staging already exists: $Stage; use a clean checkout" }
New-Item -ItemType Directory -Force -Path $Stage | Out-Null
Copy-Item (Join-Path $Bin 'photocraft.exe'), (Join-Path $Bin 'photocraft-cli.exe') $Stage
foreach ($f in 'LICENSE-APACHE', 'LICENSE-MIT', 'NOTICE', 'ATTRIBUTION.md') { Copy-Item (Join-Path $Root $f) $Stage }

# Builds made with craft-fonts embed its fonts: ship their licences, as upstream's package.ps1 does.
if ($env:CRAFT_FONTS_DIR) {
  foreach ($ofl in Get-ChildItem -Path (Join-Path $env:CRAFT_FONTS_DIR 'fonts\*\OFL.txt') -ErrorAction SilentlyContinue) {
    Copy-Item $ofl.FullName (Join-Path $Stage "OFL-$($ofl.Directory.Name).txt")
  }
}
$Zip = Join-Path $Dist "photocraft-$Version-z$Z-windows-x64-portable.zip"
Compress-Archive -Path $Stage -DestinationPath $Zip
$item = Get-Item $Zip
$sha = (Get-FileHash -Algorithm SHA256 $Zip).Hash.ToLower()
Write-Output "zip:    $($item.Name)"
Write-Output "size:   $($item.Length)"
Write-Output "sha256: $sha"
Write-Output "exe:    $Folder/photocraft.exe"

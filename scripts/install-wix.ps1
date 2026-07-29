# Pre-install WiX 3.14 for Tauri MSI bundling
# Downloads wix314-binaries.zip and extracts to ~/.tauri/WixTools314/

$ErrorActionPreference = "Stop"

$wixDir = Join-Path $env:LOCALAPPDATA "tauri\WixTools314"
$zipUrl = "https://github.com/wixtoolset/wix3/releases/download/wix3141rtm/wix314-binaries.zip"
$zipPath = Join-Path $env:TEMP "wix314-binaries.zip"

if (Test-Path (Join-Path $wixDir "candle.exe")) {
    Write-Host "WiX 3.14 already installed at $wixDir"
    exit 0
}

Write-Host "Downloading WiX 3.14 binaries..."
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

try {
    Invoke-WebRequest -Uri $zipUrl -OutFile $zipPath -UseBasicParsing
} catch {
    Write-Host "Direct download failed, trying fallback..."
    $fallbackUrl = "https://github.com/wixtoolset/wix3/releases/download/wix3141rtm/wix314-binaries.zip"
    Invoke-WebRequest -Uri $fallbackUrl -OutFile $zipPath -UseBasicParsing
}

Write-Host "Extracting to $wixDir ..."
New-Item -ItemType Directory -Force -Path $wixDir | Out-Null
Expand-Archive -Path $zipPath -DestinationPath $wixDir -Force

Remove-Item $zipPath -ErrorAction SilentlyContinue

$candle = Join-Path $wixDir "candle.exe"
if (Test-Path $candle) {
    Write-Host "WiX 3.14 installed successfully at $wixDir"
    Write-Host "candle.exe found: $candle"
} else {
    Write-Error "WiX installation failed - candle.exe not found"
    exit 1
}

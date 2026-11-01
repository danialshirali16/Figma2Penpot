param(
    [string]$Version = "0.1.0",
    [string]$Dist = "dist"
)

$ErrorActionPreference = "Stop"
New-Item -ItemType Directory -Force -Path $Dist | Out-Null

$desktop = "target/release/figma2penpot.exe"
$cli = "target/release/f2p-cli.exe"
if (-not (Test-Path $desktop)) { throw "missing $desktop; run cargo build --release -p desktop-app" }
if (-not (Test-Path $cli)) { throw "missing $cli; run cargo build --release -p f2p-cli" }

$staging = Join-Path $Dist "staging-windows"
if (Test-Path $staging) { Remove-Item -Recurse -Force $staging }
New-Item -ItemType Directory -Force -Path $staging | Out-Null
Copy-Item $desktop (Join-Path $staging "figma2penpot.exe")
Copy-Item $cli (Join-Path $staging "f2p-cli.exe")
Copy-Item "README.md" (Join-Path $staging "README.md")

$zip = Join-Path $Dist "figma2penpot-$Version-windows-x86_64.zip"
if (Test-Path $zip) { Remove-Item -Force $zip }
Compress-Archive -Path (Join-Path $staging "*") -DestinationPath $zip

$hash = Get-FileHash -Algorithm SHA256 $zip
"$($hash.Hash.ToLower())  $(Split-Path $zip -Leaf)" |
    Set-Content -Encoding ascii "$zip.sha256"
Remove-Item -Recurse -Force $staging
Write-Host "wrote $zip"

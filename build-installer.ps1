param([string]$NodeVersion = '24.22.3')
$ErrorActionPreference = 'Stop'
if ($NodeVersion -notmatch '^24\.\d+\.\d+$') { throw 'A Node 24 release version is required.' }
$runtimePath = Join-Path $PSScriptRoot 'runtime'
$archiveName = "node-v$NodeVersion-win-x64.zip"
$downloadBase = "https://nodejs.org/dist/v$NodeVersion"
$stagePath = Join-Path ([IO.Path]::GetTempPath()) ([Guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $stagePath | Out-Null
$archivePath = Join-Path $stagePath $archiveName
$checksums = (Invoke-WebRequest "$downloadBase/SHASUMS256.txt").Content
$checksumLine = $checksums -split "`n" | Where-Object { $_.TrimEnd().EndsWith("  $archiveName") }
if (@($checksumLine).Count -ne 1) { throw 'Node archive checksum is unavailable or ambiguous.' }
$expected = ($checksumLine -split '\s+')[0]
Invoke-WebRequest "$downloadBase/$archiveName" -OutFile $archivePath
if ((Get-FileHash $archivePath -Algorithm SHA256).Hash -ne $expected) { throw 'Node archive checksum mismatch.' }
Expand-Archive -LiteralPath $archivePath -DestinationPath $stagePath
New-Item -ItemType Directory -Path $runtimePath -Force | Out-Null
foreach ($name in @('node.exe', 'LICENSE')) {
    Copy-Item -LiteralPath (Join-Path $stagePath "node-v$NodeVersion-win-x64/$name") -Destination $runtimePath
}
$env:RUSTUP_TOOLCHAIN = 'stable-x86_64-pc-windows-msvc'
$env:CARGO_TARGET_DIR = Join-Path $PSScriptRoot 'target/installer'
$env:PATH = (Join-Path $env:USERPROFILE '.cargo/bin') + ';' + $env:PATH
Push-Location $PSScriptRoot
try {
    & pnpm dlx @tauri-apps/cli@2.12.6 build --target x86_64-pc-windows-msvc --bundles nsis
    if ($LASTEXITCODE -ne 0) { throw "Tauri build failed: $LASTEXITCODE" }
} finally { Pop-Location }

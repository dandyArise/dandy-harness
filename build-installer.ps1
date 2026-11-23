param([string]$NodeVersion = '24.22.3', [string]$EngineDirectory,
    [ValidateSet('lzma', 'zlib', 'bzip2', 'none')][string]$Compression = 'lzma')
$ErrorActionPreference = 'Stop'
if (-not $EngineDirectory) {
    & (Join-Path $PSScriptRoot 'prepare-engine.ps1')
    $EngineDirectory = (Get-Content -LiteralPath (Join-Path $PSScriptRoot 'target/engine-ready.txt') -Raw).Trim()
}
$EngineDirectory = (Resolve-Path -LiteralPath $EngineDirectory).Path
if (-not (Test-Path (Join-Path $EngineDirectory 'node_modules/@deepseek-ai/dsh/lib/bin.js'))) {
    throw 'Le moteur de production est incomplet.'
}
& node (Join-Path $PSScriptRoot 'smoke-engine.mjs') $EngineDirectory
if ($LASTEXITCODE -ne 0) { throw 'Le test web du moteur isolé a échoué.' }
$resourceConfig = Join-Path $PSScriptRoot 'target/installer-resources.json'
@{ bundle = @{ resources = @{ ($EngineDirectory + '/') = 'engine/' }; windows = @{ nsis = @{ compression = $Compression } } } } | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath $resourceConfig
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
    & pnpm dlx @tauri-apps/cli@2.12.6 build --target x86_64-pc-windows-msvc --bundles nsis --config $resourceConfig
    if ($LASTEXITCODE -ne 0) { throw "Tauri build failed: $LASTEXITCODE" }
} finally { Pop-Location }

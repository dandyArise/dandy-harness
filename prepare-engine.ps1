param([string]$Source = 'D:\Github\Projets\IA\deepseek-harness')
$ErrorActionPreference = 'Stop'
$Source = (Resolve-Path -LiteralPath $Source).Path
$env:DSH_BUILD_CLIENT_PROFILE = 'local'
$env:npm_config_verify_deps_before_run = 'false'
$env:npm_execpath = Join-Path (Split-Path (Split-Path (Get-Command node).Source)) 'node_modules/pnpm/bin/pnpm.mjs'
if (-not (Test-Path -LiteralPath $env:npm_execpath)) { throw 'pnpm.mjs introuvable à côté de Node.' }
function Check-Exit { if ($LASTEXITCODE -ne 0) { throw "Préparation du moteur : code $LASTEXITCODE" } }
Push-Location $Source
try {
    & pnpm --config.verify-deps-before-run=false run build:artifacts
    Check-Exit
    & node --import tsx scripts/release/pack.ts --family vendor --out (Join-Path $PSScriptRoot 'target/packed-vendor') --concurrency 1
    Check-Exit
    & node --import tsx apps/desktop/scripts/pack-local-packages.ts --out (Join-Path $PSScriptRoot 'target/packed-dsh') --concurrency 1
    Check-Exit
    & pnpm --config.verify-deps-before-run=false --dir native/system/packages/entry pack --pack-destination (Join-Path $PSScriptRoot 'target/packed-vendor')
    Check-Exit
} finally { Pop-Location }
$output = Join-Path $PSScriptRoot ('target/engine-' + [Guid]::NewGuid().ToString('N'))
& node (Join-Path $PSScriptRoot 'prepare-engine.mjs') $Source $output
Check-Exit
Push-Location $output
try {
    & pnpm install --prod --config.verify-deps-before-run=false
    Check-Exit
    & node node_modules/@deepseek-ai/dsh/lib/bin.js --help
    Check-Exit
} finally { Pop-Location }
Set-Content -LiteralPath (Join-Path $PSScriptRoot 'target/engine-ready.txt') -Value $output

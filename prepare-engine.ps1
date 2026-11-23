param([string]$Source = 'D:\Github\Projets\IA\deepseek-harness')
$ErrorActionPreference = 'Stop'
$Source = (Resolve-Path -LiteralPath $Source).Path
$env:DSH_BUILD_CLIENT_PROFILE = 'local'
$env:npm_config_verify_deps_before_run = 'false'
function Check-Exit { if ($LASTEXITCODE -ne 0) { throw "Préparation du moteur : code $LASTEXITCODE" } }
Push-Location $Source
try {
    & pnpm --config.verify-deps-before-run=false run build:artifacts
    Check-Exit
} finally { Pop-Location }
Push-Location $PSScriptRoot
try {
    & node (Join-Path $PSScriptRoot 'pack-engine.mjs') $Source
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

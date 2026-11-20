param(
  [Parameter(Mandatory=$true)][string]$Exe,
  [Parameter(Mandatory=$true)][string]$Output,
  [string]$Label = 'baseline',
  [int]$Runs = 3,
  [switch]$Serial
)
$ErrorActionPreference = 'Stop'
$rows = @()
for ($run = 0; $run -lt $Runs; $run++) {
  if (Get-NetTCPConnection -LocalPort 18643 -State Listen -ErrorAction SilentlyContinue) { throw 'Harness is already running on port 18643' }
  $phaseFile = Join-Path $env:TEMP ('dandy-startup-' + [guid]::NewGuid() + '.txt')
  $oldProbe = $env:DANDY_STARTUP_PROBE
  $oldSerial = $env:DANDY_STARTUP_SERIAL
  $env:DANDY_STARTUP_PROBE = $phaseFile
  $env:DANDY_STARTUP_SERIAL = if ($Serial) { '1' } else { '0' }
  $wallStarted = [DateTime]::UtcNow
  $watch = [Diagnostics.Stopwatch]::StartNew()
  $app = Start-Process -FilePath $Exe -WorkingDirectory (Split-Path (Split-Path (Split-Path (Split-Path $Exe)))) -PassThru
  $env:DANDY_STARTUP_PROBE = $oldProbe
  $env:DANDY_STARTUP_SERIAL = $oldSerial
  $ready = $null
  $samples = @()
  $owned = [Collections.Generic.HashSet[int]]::new()
  [void]$owned.Add($app.Id)
  try {
    while ($watch.Elapsed.TotalSeconds -lt 60) {
      $processes = @(Get-CimInstance Win32_Process | Select-Object ProcessId,ParentProcessId,Name)
      do {
        $added = $false
        foreach ($process in $processes) {
          if ($owned.Contains([int]$process.ParentProcessId) -and $owned.Add([int]$process.ProcessId)) { $added = $true }
        }
      } while ($added)
      $groups = @{}
      foreach ($process in $processes) {
        if (-not $owned.Contains([int]$process.ProcessId)) { continue }
        $live = Get-Process -Id $process.ProcessId -ErrorAction SilentlyContinue
        if (-not $live) { continue }
        $kind = switch -Wildcard ($process.Name) { 'dandy-harness*' {'tauri'} 'node.exe' {'node'} 'msedgewebview2.exe' {'webview'} default {'other'} }
        if (-not $groups.ContainsKey($kind)) { $groups[$kind] = @{privateBytes=0L;workingSet=0L;count=0} }
        $groups[$kind].privateBytes += $live.PrivateMemorySize64
        $groups[$kind].workingSet += $live.WorkingSet64
        $groups[$kind].count++
      }
      $app.Refresh()
      if ($null -eq $ready -and $app.MainWindowTitle -eq 'DANDY-BENCH-CHAT-READY') { $ready = $watch.Elapsed.TotalMilliseconds }
      $samples += @{ms=$watch.Elapsed.TotalMilliseconds;groups=$groups}
      if ($null -ne $ready -and $watch.Elapsed.TotalMilliseconds -gt $ready + 5000) { break }
      if ($app.HasExited) { throw 'Harness exited during benchmark' }
      Start-Sleep -Milliseconds 100
    }
    if ($null -eq $ready) { throw 'No editable composer detected within 60 s' }
    $phases = @{}
    foreach ($line in (Get-Content -LiteralPath $phaseFile)) {
      $pair = $line.Split('='); $phases[$pair[0]] = [double]::Parse($pair[1], [Globalization.CultureInfo]::InvariantCulture)
    }
    $launchOffset = ($app.StartTime.ToUniversalTime() - $wallStarted).TotalMilliseconds
    $rows += @{label=$Label;run=$run;readyMs=$phases.chat_ready;observedReadyMs=$ready;launchOffsetMs=$launchOffset;phases=$phases;samples=$samples;rootPid=$app.Id}
    Write-Output "$Label run $run : chat $([math]::Round($phases.chat_ready)) ms; engine $($phases.engine_ready) ms"
  } finally {
    # Only the benchmark-owned Tauri root; its Job Object owns the engine tree.
    Stop-Process -Id $app.Id -ErrorAction SilentlyContinue
    Start-Sleep -Milliseconds 1200
    $survivors = @(Get-Process -Id @($owned) -ErrorAction SilentlyContinue)
    if ($survivors.Count) { Write-Warning "Survivors after root shutdown: $($survivors.Id -join ',')" }
    if (Test-Path -LiteralPath $phaseFile) { Remove-Item -LiteralPath $phaseFile }
  }
}
$rows | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $Output -Encoding utf8

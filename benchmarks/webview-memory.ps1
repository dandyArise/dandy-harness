param(
  [Parameter(Mandatory=$true)][string]$Output,
  [Parameter(Mandatory=$true)][string]$Label,
  [int]$Samples = 5
)
$ErrorActionPreference = 'Stop'
$roots = @(Get-Process -Name dandy-harness -ErrorAction Stop)
if ($roots.Count -ne 1) { throw 'Expected exactly one Dandy Harness process' }
$rootId = $roots[0].Id
$records = @()
for ($i = 0; $i -lt $Samples; $i++) {
  $all = @(Get-CimInstance Win32_Process)
  $owned = [Collections.Generic.HashSet[int]]::new()
  [void]$owned.Add($rootId)
  do {
    $added = $false
    foreach ($p in $all) {
      if ($owned.Contains([int]$p.ParentProcessId) -and $owned.Add([int]$p.ProcessId)) { $added = $true }
    }
  } while ($added)
  $rows = @($all | Where-Object { $owned.Contains([int]$_.ProcessId) } | ForEach-Object {
    $live = Get-Process -Id $_.ProcessId -ErrorAction SilentlyContinue
    if ($live) {
      $kind = if ($_.CommandLine -match '--type=([^ ]+)') { $Matches[1] } else { 'main' }
      [pscustomobject]@{ name=$_.Name; kind=$kind; privateBytes=$live.PrivateMemorySize64; workingSet=$live.WorkingSet64 }
    }
  })
  $web = @($rows | Where-Object name -eq 'msedgewebview2.exe')
  $records += [pscustomobject]@{
    label=$Label; sample=$i; at=[DateTime]::UtcNow.ToString('o'); processes=$rows
    webPrivate=($web | Measure-Object privateBytes -Sum).Sum
    webWorkingSet=($web | Measure-Object workingSet -Sum).Sum
    totalPrivate=($rows | Measure-Object privateBytes -Sum).Sum
  }
  Start-Sleep -Seconds 1
}
$records | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath $Output -Encoding utf8
$records | Select-Object label,sample,@{n='WebPrivateMiB';e={[math]::Round($_.webPrivate/1MB,1)}},@{n='WebWorkingSetMiB';e={[math]::Round($_.webWorkingSet/1MB,1)}} | Format-Table

# Bounded offline loader probe (issue #289). Inputs (globals, optional):
#   $ptag     run label                      (default 'A')
#   $plisten  UnityLogListening true/false   (default $false)
#   $prt      directory with replacement core DLLs to overlay (default none)
#   $pwait    seconds to keep the client alive after chainloader start (default 45)
#   $pplug    directory with plugin DLLs to install (default none)
#   $psteps   comma separated probe steps written to plugins\steps.txt
#   $ppreload PreloadIL2CPPInteropAssemblies (default false)
#   $pdone    seconds to keep the client running until the plugin writes plugins\probe.done
#             (an interactive in-world session; replaces $pwait)
$ErrorActionPreference = 'Stop'
if (-not $ptag) { $ptag = 'A' }
if ($null -eq $plisten) { $plisten = $false }
if (-not $pwait) { $pwait = 45 }
$rust = 'C:\Program Files (x86)\Steam\steamapps\common\Rust'
$dl = "$HOME\Downloads"
$probe = "$dl\claude-loader-probe"
$zip = "$dl\BepInEx-Unity.IL2CPP-win-x64-6.0.0-be.788+5b766a3.zip"
# The interop set must have been generated for the installed Steam build. A set from another build
# resolves wrappers to the wrong native methods and crashes (issue #289, 9 October 2026), so there
# is no fallback: generate first with `zz m`.
$pb = ((gc 'C:\Program Files (x86)\Steam\steamapps\appmanifest_252490.acf' | sls '"buildid"').Line -split '"')[3]
$gen = "$dl\claude-loader-probe\gen-$pb\out"
if (-not ((Test-Path "$gen\Assembly-CSharp.dll") -and (sls -Path "$dl\claude-loader-probe\gen-$pb\gen.log" -Pattern 'GEN DONE' -Quiet))) { Write-Host "ABORT: no completed interop set for installed build $pb; run zz m first"; return }
$run = "$probe\run-$ptag-" + (Get-Date -Format 'HHmmss')
$roots = 'BepInEx','dotnet','winhttp.dll','doorstop_config.ini','.doorstop_version','changelog.txt'
function say($m) { Write-Host $m }
if (Get-Process RustClient -ErrorAction SilentlyContinue) { say 'ABORT: RustClient is already running'; return }
$present = $roots | ? { Test-Path (Join-Path $rust $_) }
if ($present) { say ('ABORT: bootstrap roots already present: ' + ($present -join ',')); return }
if (-not (Get-Process steam -ErrorAction SilentlyContinue)) { say 'ABORT: Steam is not running'; return }
$before = (ls $rust -Force | % Name | sort) -join '|'
New-Item -ItemType Directory -Force $run | Out-Null
$stage = "$probe\bep788"
if (-not (Test-Path "$stage\winhttp.dll")) { Expand-Archive -LiteralPath $zip -DestinationPath $stage -Force }
$stRoots = ls $stage -Force | % Name
say ('stage roots: ' + ($stRoots -join ','))
try {
  foreach ($n in $stRoots) { Copy-Item -LiteralPath (Join-Path $stage $n) -Destination $rust -Recurse }
  $bx = Join-Path $rust 'BepInEx'
  New-Item -ItemType Directory -Force "$bx\interop","$bx\config","$bx\plugins" | Out-Null
  Copy-Item "$gen\*" "$bx\interop"
  say ('interop files: ' + (ls "$bx\interop").Count + ' from ' + $gen.Replace($dl,'~'))
  if ($prt) { Copy-Item "$prt\*" "$bx\core" -Force; say ('overlay core: ' + ((ls $prt | % Name) -join ',')) }
  if ($psteps) { Set-Content "$bx\plugins\steps.txt" $psteps }
  if ($pplug) { Copy-Item "$pplug\*" "$bx\plugins" -Force; say ('plugins: ' + ((ls $pplug | % Name) -join ',')) }
  $cfg = "[IL2CPP]`r`nUpdateInteropAssemblies = false`r`nPreloadIL2CPPInteropAssemblies = " + ([bool]$ppreload).ToString().ToLower() + "`r`n`r`n[Logging]`r`nUnityLogListening = " + $plisten.ToString().ToLower() + "`r`n`r`n[Logging.Disk]`r`nLogLevels = All`r`n`r`n[Logging.Console]`r`nEnabled = false`r`n"
  [IO.File]::WriteAllText("$bx\config\BepInEx.cfg", $cfg)
  $p = Start-Process -FilePath (Join-Path $rust 'RustClient.exe') -WorkingDirectory $rust -PassThru
  $log = "$bx\LogOutput.log"
  $t0 = Get-Date; $seen = $null; $state = 'timeout-before-chainloader'
  $limit = 240; if ($pdone) { $limit = [int]$pdone }
  while (((Get-Date) - $t0).TotalSeconds -lt $limit) {
    Start-Sleep 3
    if ($p.HasExited) { $state = 'exited code=' + $p.ExitCode + ' at ' + [int]((Get-Date) - $t0).TotalSeconds + 's'; break }
    if (-not $seen -and (Test-Path $log)) {
      $txt = Get-Content $log -Raw -ErrorAction SilentlyContinue
      if ($txt -match 'Chainloader startup complete') { $seen = Get-Date }
    }
    if ($pdone -and (Test-Path "$bx\plugins\probe.done")) { $state = 'plugin reported done at ' + [int]((Get-Date) - $t0).TotalSeconds + 's; responding=' + $p.Responding; break }
    if (-not $pdone -and $seen -and ((Get-Date) - $seen).TotalSeconds -ge $pwait) { $state = 'alive ' + $pwait + 's after chainloader; responding=' + $p.Responding; break }
  }
  say ("RESULT[$ptag]: " + $state + '; chainloaderSeen=' + [bool]$seen)
  if (-not $p.HasExited) { $p.CloseMainWindow() | Out-Null; Start-Sleep 5; if (-not $p.HasExited) { Stop-Process -Id $p.Id -Force }; Start-Sleep 3 }
  Get-Process UnityCrashHandler64 -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
} finally {
  Start-Sleep 2
  $bx = Join-Path $rust 'BepInEx'
  if (Test-Path "$bx\plugins\probe.log") { Copy-Item "$bx\plugins\probe.log" $run }
  foreach ($f in 'LogOutput.log','ErrorLog.log') { if (Test-Path "$bx\$f") { Copy-Item "$bx\$f" $run } }
  ls $rust -Filter 'preloader_*.log' -ErrorAction SilentlyContinue | % { Copy-Item $_.FullName $run }
  $pl = "$HOME\AppData\LocalLow\Facepunch Studios LTD\Rust\Player.log"
  if (Test-Path $pl) { Copy-Item $pl $run }
  New-Item -ItemType Directory -Force "$run\removed" | Out-Null
  foreach ($n in ($roots + (ls $rust -Filter 'preloader_*.log' -ErrorAction SilentlyContinue | % Name))) {
    $src = Join-Path $rust $n
    if (Test-Path -LiteralPath $src) { Move-Item -LiteralPath $src -Destination "$run\removed" }
  }
  $after = (ls $rust -Force | % Name | sort) -join '|'
  say ('rollback clean=' + ($before -eq $after))
  if ($before -ne $after) { say "before=$before"; say "after=$after" }
}
$global:plast = $run
say "run dir: $run"
if (Test-Path "$run\LogOutput.log") {
  $l = Get-Content "$run\LogOutput.log"
  say ('log lines=' + $l.Count)
  $l | ? { $_ -notmatch 'DobbyDetour|NativeDetour' } | select -Last $(if ($ptail) { $ptail } else { 26 }) | % { if ($_.Length -gt 1500) { $_.Substring(0,1500) } else { $_ } }
} else { say 'no LogOutput.log' }
if (Test-Path "$run\probe.log") { say '--- probe.log'; Get-Content "$run\probe.log" | % { if ($_.Length -gt 1900) { $_.Substring(0,1900) } else { $_ } } }
if (Test-Path "$run\ErrorLog.log") { say '--- ErrorLog'; Get-Content "$run\ErrorLog.log" | select -First 12 }

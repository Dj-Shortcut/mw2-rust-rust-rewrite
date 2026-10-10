# Check the launcher's safety net without touching the game by hand: start the desktop launcher,
# wait for Rust, kill the launcher as if its window had been closed, close Rust, and see whether
# the hidden helper takes the loader out of the Rust folder.
$probe = "$HOME\Downloads\claude-loader-probe"; $rust = 'C:\Program Files (x86)\Steam\steamapps\common\Rust'
$roots = 'BepInEx','dotnet','winhttp.dll','doorstop_config.ini','.doorstop_version','changelog.txt'
function present { @($roots | ? { Test-Path -LiteralPath (Join-Path $rust $_) }) -join ',' }
function procs($rx) { @(Get-CimInstance Win32_Process | ? { $_.CommandLine -match $rx }) }
if (Get-Process RustClient -ErrorAction SilentlyContinue) { 'ABORT: Rust is running'; return }
$f = Join-Path ([Environment]::GetFolderPath('Desktop')) 'Rust Skate.cmd'
if (-not (Test-Path $f)) { 'ABORT: no launcher on the desktop'; return }
Remove-Item "$probe\launcher\watch-last.txt" -ErrorAction SilentlyContinue
Start-Process $f
$t0 = Get-Date
while (-not (Get-Process RustClient -ErrorAction SilentlyContinue) -and ((Get-Date) - $t0).TotalSeconds -lt 120) { Start-Sleep 2 }
if (-not (Get-Process RustClient -ErrorAction SilentlyContinue)) { 'ABORT: Rust did not start; loader files: ' + (present); return }
'1 rust started after ' + [int]((Get-Date) - $t0).TotalSeconds + ' s; loader files: ' + (present)
'2 helper running: ' + (procs 'launcher\\watch\.ps1').Count + ' launcher running: ' + (procs 'launcher\\skate\.ps1').Count
foreach ($p in (procs 'launcher\\skate\.ps1')) {
  $parent = Get-Process -Id $p.ParentProcessId -ErrorAction SilentlyContinue
  Stop-Process -Id $p.ProcessId -Force -ErrorAction SilentlyContinue
  if ($parent -and $parent.ProcessName -eq 'cmd') { Stop-Process -Id $parent.Id -Force -ErrorAction SilentlyContinue }
}
Start-Sleep 15
'3 launcher killed; launcher running: ' + (procs 'launcher\\skate\.ps1').Count + ' helper running: ' + (procs 'launcher\\watch\.ps1').Count + ' loader files: ' + (present)
$r = Get-Process RustClient -ErrorAction SilentlyContinue | select -First 1
if ($r) { [void]$r.CloseMainWindow(); $t1 = Get-Date; while (-not $r.HasExited -and ((Get-Date) - $t1).TotalSeconds -lt 30) { Start-Sleep 2 }; if (-not $r.HasExited) { Stop-Process -Id $r.Id -Force } }
Get-Process UnityCrashHandler64 -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
'4 rust closed; loader files right after: ' + (present)
$t2 = Get-Date
while ((present) -and ((Get-Date) - $t2).TotalSeconds -lt 90) { Start-Sleep 5 }
'5 after ' + [int]((Get-Date) - $t2).TotalSeconds + ' s: loader files: ' + (present) + ' (empty means the helper cleaned up)'
'6 helper note: ' + $(if (Test-Path "$probe\launcher\watch-last.txt") { (Get-Content "$probe\launcher\watch-last.txt") -join ' | ' } else { 'none written' })

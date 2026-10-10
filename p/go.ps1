# Start the owner's desktop launcher the way a double click would (for checking it after `zz inst`).
$f = Join-Path ([Environment]::GetFolderPath('Desktop')) 'Rust Skate.cmd'
if (-not (Test-Path $f)) { 'no launcher on the desktop; run zz inst first'; return }
Start-Process $f
'started: ' + $f

# Safety net of the owner's launcher: waits for Rust to start and to close, gives the launcher
# time to tidy up itself, and then moves any loader files still in the Rust folder out of it.
$L = "$HOME\Downloads\claude-loader-probe\launcher"
$t0 = Get-Date
while (-not (Get-Process RustClient -ErrorAction SilentlyContinue) -and ((Get-Date) - $t0).TotalSeconds -lt 300) { Start-Sleep 3 }
while (Get-Process RustClient -ErrorAction SilentlyContinue) { Start-Sleep 5 }
Start-Sleep 30
. "$L\clean.ps1" | Out-File "$L\watch-last.txt"

# Take a loader that a session left behind out of the Rust folder (for example when the window was
# closed before the session could tidy up). Moves the files to the staging folder; deletes nothing.
$rust = 'C:\Program Files (x86)\Steam\steamapps\common\Rust'
$probe = "$HOME\Downloads\claude-loader-probe"
if (Get-Process RustClient -ErrorAction SilentlyContinue) { 'ABORT: RustClient is running; quit Rust first'; return }
$roots = 'BepInEx','dotnet','winhttp.dll','doorstop_config.ini','.doorstop_version','changelog.txt'
$found = @($roots + (ls $rust -Filter 'preloader_*.log' -ErrorAction SilentlyContinue | % Name) | ? { Test-Path -LiteralPath (Join-Path $rust $_) })
if (-not $found) { 'clean: no loader files in the Rust folder'; return }
$dst = "$probe\removed-" + (Get-Date -Format 'yyyyMMdd-HHmmss')
New-Item -ItemType Directory -Force $dst | Out-Null
foreach ($n in $found) { Move-Item -LiteralPath (Join-Path $rust $n) -Destination $dst }
'moved out: ' + ($found -join ',')
'left behind: ' + ((@($roots | ? { Test-Path -LiteralPath (Join-Path $rust $_) }) -join ',') + ' (empty means clean)')

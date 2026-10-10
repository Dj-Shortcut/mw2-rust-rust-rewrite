# Take a loader that a session left behind out of the Rust folder (for example when the window was
# closed before the session could tidy up). Moves the files to the staging folder; deletes nothing.
# Only a loader that these tools put there is touched: session.ps1 leaves a note in the staging
# folder for as long as its loader is in the Rust folder.
$rust = 'C:\Program Files (x86)\Steam\steamapps\common\Rust'
$probe = "$HOME\Downloads\claude-loader-probe"
$mark = "$probe\loader-in-rust-folder.txt"
if (Get-Process RustClient -ErrorAction SilentlyContinue) { 'ABORT: RustClient is running; quit Rust first'; return }
$roots = 'BepInEx','dotnet','winhttp.dll','doorstop_config.ini','.doorstop_version','changelog.txt'
$found = @($roots + (ls $rust -Filter 'preloader_*.log' -ErrorAction SilentlyContinue | % Name) | ? { Test-Path -LiteralPath (Join-Path $rust $_) })
if (-not $found) { Remove-Item -LiteralPath $mark -ErrorAction SilentlyContinue; 'clean: no loader files in the Rust folder'; return }
if (-not (Test-Path -LiteralPath $mark)) { 'ABORT: the Rust folder holds mod loader files that these tools did not put there: ' + ($found -join ',') + '. Nothing was changed.'; return }
$dst = "$probe\removed-" + (Get-Date -Format 'yyyyMMdd-HHmmss')
New-Item -ItemType Directory -Force $dst | Out-Null
foreach ($n in $found) { Move-Item -LiteralPath (Join-Path $rust $n) -Destination $dst }
$left = @($roots | ? { Test-Path -LiteralPath (Join-Path $rust $_) })
if (-not $left) { Remove-Item -LiteralPath $mark -ErrorAction SilentlyContinue }
'moved out: ' + ($found -join ',')
'left behind: ' + (($left -join ',') + ' (empty means clean)')

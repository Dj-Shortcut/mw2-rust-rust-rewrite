# The owner's launcher, installed by launcher.ps1 as launcher\skate.ps1 and started from the desktop
# file "Rust Skate.cmd". It puts the mod loader and the released plugin into the Rust folder, starts
# Rust through the Steam shortcut "RustClient" (so that the controller layout of that shortcut
# applies), waits until Rust is closed and takes everything out of the Rust folder again.
# Nothing is downloaded. The window must stay open while playing.
$probe = "$HOME\Downloads\claude-loader-probe"; $L = "$probe\launcher"
$rust = 'C:\Program Files (x86)\Steam\steamapps\common\Rust'
function line($m) { Write-Host $m }
line ''
line '  RUST SKATE'
line '  ----------'
if (Test-Path "$L\version.txt") { line ('  ' + (Get-Content "$L\version.txt" -First 1)) }
line ''
if (Get-Process RustClient -ErrorAction SilentlyContinue) { line '  Rust is already running. Quit Rust first, then start this again.'; return }
if (-not (Test-Path "$probe\play\ShortcutSkateClient.dll")) { line '  The skate plugin is not installed here. Ask Claude to run the install step again.'; return }
# A session that was cut off (window closed, PC stopped) can have left the loader in the Rust folder.
. "$L\clean.ps1" | % { line "  $_" }
$there = @('BepInEx','dotnet','winhttp.dll','doorstop_config.ini','.doorstop_version','changelog.txt' | ? { Test-Path -LiteralPath (Join-Path $rust $_) })
if ($there) {
  line '  The Rust folder already holds a mod loader that this launcher did not put there.'
  line '  Nothing was started and nothing was changed.'
  return
}
$pb = ((Get-Content 'C:\Program Files (x86)\Steam\steamapps\appmanifest_252490.acf' | Select-String '"buildid"').Line -split '"')[3]
if (-not ((Test-Path "$probe\gen-$pb\out\Assembly-CSharp.dll") -and (Select-String -Path "$probe\gen-$pb\gen.log" -Pattern 'GEN DONE.* errors=0( |$)' -Quiet))) {
  line "  Rust has been updated (build $pb) and the mod has not been prepared for this version yet."
  line '  Nothing was started and nothing was changed. Ask Claude to prepare the new version.'
  return
}
if (-not (Get-Process steam -ErrorAction SilentlyContinue)) { line '  Steam is not running. Start Steam, then start this again.'; return }
line '  Starting Rust with the skate mod. In the main menu pick your own server under QUICK JOIN.'
line '  On the server: K or two quick jumps gets you on the board.'
line '  Keep this window open. When you quit Rust, the mod is removed from the Rust folder again.'
line ''
# A second, hidden helper takes the loader out after Rust closes even if this window is closed first.
Start-Process powershell -WindowStyle Hidden -ArgumentList '-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', "`"$L\watch.ps1`""
$global:ptag = 'P'; $global:plisten = $false; $global:prt = "$probe\rt"; $global:pplug = "$probe\play"; $global:psteps = 'skate'
$global:pwait = 30; $global:pdone = 43200; $global:ppreload = $false; $global:ptail = 3; $global:psteam = $true; $global:pshortcut = $null; $global:pconnect = $false
. "$L\session.ps1"
line ''
$left = @('BepInEx','dotnet','winhttp.dll','doorstop_config.ini','.doorstop_version','changelog.txt' | ? { Test-Path -LiteralPath (Join-Path $rust $_) })
if ($left) { line ('  WARNING: still in the Rust folder: ' + ($left -join ', ')); line '  Start "Rust Skate - remove mod files.cmd" on the desktop before playing Rust normally.' }
else { line '  Done. The Rust folder is back to normal.' }

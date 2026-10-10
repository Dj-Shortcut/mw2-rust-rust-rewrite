# As play.ps1, but started through the owner's Steam shortcut "RustClient", so that the Steam
# Input layout made for that shortcut applies and the controller works.
$global:ptag='S'; $global:psteps='skate'; $global:pdone=3600; $global:psteam=$true; $global:pconnect=$false; $global:pkeep=$false; $global:plisten=$false; $global:prt="$HOME\Downloads\claude-loader-probe\rt"; $global:pplug="$HOME\Downloads\claude-loader-probe\plugin"; $global:pwait=30; $global:ppreload=$false; $global:ptail=4
# Test build: the reader outside the game runs beside the session, as a process of its own.
$probe = "$HOME\Downloads\claude-loader-probe"
[IO.File]::WriteAllText("$probe\sticks.ps1", ("" + (zg 'sticks.ps1')), (New-Object Text.UTF8Encoding($false)))
$env:SKATE_PAD_SERVE = 'RustClient'
Start-Process powershell -WindowStyle Hidden -ArgumentList '-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', "`"$probe\sticks.ps1`""
$env:SKATE_PAD_SERVE = $null
$global:pnext='session'; zz build

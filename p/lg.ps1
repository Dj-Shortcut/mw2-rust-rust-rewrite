# Filtered view of the last run's probe.log: keep lines matching $pinc (default all), drop lines
# matching $pexc (default none), then page the result with `zz more`.
$src = "$plast\probe.log"
if (-not (Test-Path $src)) { 'no probe.log in the last run'; return }
$inc = '.'; if ($pinc) { $inc = $pinc }
$l = Get-Content $src | ? { $_ -match $inc }
if ($pexc) { $l = $l | ? { $_ -notmatch $pexc } }
$view = "$HOME\Downloads\claude-loader-probe\view.txt"
$l | Out-File $view -Encoding utf8
'lines=' + @($l).Count
$global:pfile = $view; $global:poff = 0
zz more

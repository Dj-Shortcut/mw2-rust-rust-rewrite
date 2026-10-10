# Another file of the last session (default the game's own Player.log; set $pname for another):
# the last 60 lines matching $pinc, cut to 300 characters each.
$n = 'Player.log'; if ($pname) { $n = $pname }
$src = "$plast\$n"
if (-not (Test-Path $src)) { "no $n in the last session"; return }
$inc = '.'; if ($pinc) { $inc = $pinc }
$l = @(Get-Content $src | ? { $_ -match $inc })
'matches=' + $l.Count + ' in ' + $n
$l | select -Last 60 | % { if ($_.Length -gt 300) { $_.Substring(0,300) } else { $_ } }

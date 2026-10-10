# What the right stick did in the last session, one row for each movement the plugin logged
# ("PAD stick ..."): what came of it and the shape of the movement, so that a flick that did nothing
# or the wrong thing can be picked out and replayed offline. Page with `zz more`.
#   $pshow = n   print movement n as it was logged, with every point, instead of the table
# A row: number, where the rider was (G ground, A air, R grind, B bail, F on foot), how long, what
# came of it, then the zones the stick went through in order: B held back and F held forward with
# the milliseconds it stayed (and @ the angle it arrived at when that was not straight), r the rim
# elsewhere with its angle from straight ahead, to the left negative, and > the milliseconds
# between two zones; m marks time held part of the way back, where a manual is.
$src = "$plast\skate.log"
if (-not (Test-Path $src)) { 'no skate.log in the last session'; return }
$rows = @(Get-Content $src | ? { $_ -match '^PAD stick ' })
if ($rows.Count -eq 0) { 'the right stick was not moved in the last session'; return }
if ($pshow) {
  $n = [int]$pshow
  if ($n -lt 1 -or $n -gt $rows.Count) { "there are $($rows.Count) movements"; return }
  $rows[$n - 1]
  return
}
function zone($x, $y) {
  $far = [Math]::Sqrt($x * $x + $y * $y)
  if ($far -ge 70 -and [Math]::Atan2([Math]::Abs($x), -$y) * 180 / [Math]::PI -le 40) { return 'B' }
  if ($far -ge 70 -and [Math]::Atan2([Math]::Abs($x), $y) * 180 / [Math]::PI -le 40) { return 'F' }
  if ($far -ge 80) { return 'r' }
  return '.'
}
$out = @(); $kinds = @{}; $i = 0
foreach ($row in $rows) {
  $i++
  if ($row -notmatch '^PAD stick (\d+) ms, (.+?) -> (.+?) \|(.*)$') { $out += "$i ?"; continue }
  $ms = $Matches[1]; $where = $Matches[2]; $came = $Matches[3]; $path = $Matches[4].Trim()
  $letter = switch -Wildcard ($where) { 'Ground' { 'G' } 'Air' { 'A' } 'Grind' { 'R' } 'Bail' { 'B' } default { 'F' } }
  $kind = "$letter>" + ($came -replace ' pop=.*', '')
  $kinds[$kind] = 1 + [int]$kinds[$kind]
  $pts = @($path -split ' ' | ? { $_ } | % { $a = $_ -split ':'; $xy = $a[1] -split ','; , @([int]$a[0], [int]$xy[0], [int]$xy[1]) })
  $shape = ''; $last = ''; $since = 0; $manual = 0; $at = ''
  for ($k = 0; $k -lt $pts.Count; $k++) {
    $t = $pts[$k][0]; $x = $pts[$k][1]; $y = $pts[$k][2]
    $until = if ($k + 1 -lt $pts.Count) { $pts[$k + 1][0] } else { $t }
    if (-$y -ge 25 -and -$y -le 62 -and [Math]::Abs($x) -le 35) { $manual += $until - $t }
    $z = zone $x $y
    if ($z -eq $last) { continue }
    if ($last -eq 'B' -or $last -eq 'F') { $shape += "$($t - $since)$at" }
    # The way between two zones, in milliseconds: how fast the flick was.
    if ($last -eq '.' -and $z -ne '.' -and $shape -ne '') { $shape += " >$($t - $since)" }
    $last = $z; $since = $t; $at = ''
    if ($z -eq '.') { continue }
    # The angle the stick arrived at: from straight ahead for the rim and the forward zone, from
    # straight back for the back zone; to the left negative.
    $angle = [int]([Math]::Atan2($x, $(if ($z -eq 'B') { -$y } else { $y })) * 180 / [Math]::PI)
    if ($z -eq 'r') { $shape += " r$angle" } else { $shape += " $z"; if ([Math]::Abs($angle) -ge 10) { $at = "@$angle" } }
  }
  if ($last -eq 'B' -or $last -eq 'F') { $shape += "$($pts[$pts.Count - 1][0] - $since)$at" }
  if ($manual -ge 100) { $shape += " m$manual" }
  $out += ("{0} {1} {2}ms {3} |{4}" -f $i, $letter, $ms, $came, $shape)
}
$view = "$HOME\Downloads\claude-loader-probe\view.txt"
$head = 'movements=' + $rows.Count + ': ' + (($kinds.Keys | sort | % { $_ + ' ' + $kinds[$_] }) -join ', ')
(@($head) + $out) | Out-File $view -Encoding utf8
$global:pfile = $view; $global:poff = 0
zz more

# Makes the newest session folder the one that log, logall and file show, and sums up its ride.
# The owner's launcher runs in a window of its own, so its sessions are not this window's last one.
$probe = "$HOME\Downloads\claude-loader-probe"
$run = Get-ChildItem $probe -Directory -Filter 'run-*' -ErrorAction SilentlyContinue | Sort-Object CreationTime | Select-Object -Last 1
if (-not $run) { 'no session folder yet'; return }
$global:plast = $run.FullName
'last session: ' + $run.Name + ', started ' + $run.CreationTime.ToString('yyyy-MM-dd HH:mm:ss') + ' | ' + ((Get-ChildItem $run.FullName -File | % { "{0} {1}" -f $_.Name, $_.Length }) -join ', ')
$log = Join-Path $run.FullName 'skate.log'
if (-not (Test-Path $log)) { 'no skate.log in it'; return }
$l = @(Get-Content $log)
function most($pattern, $lowest) {
  $v = @($l | % { if ($_ -match $pattern) { [double]$Matches[1] } })
  if ($v.Count -eq 0) { return 'none' }
  $m = $v | Measure-Object -Minimum -Maximum
  if ($lowest) { $m.Minimum } else { $m.Maximum }
}
function count($pattern) { @($l | ? { $_ -match $pattern }).Count }
'lines=' + $l.Count + ' mounts=' + (count '^SKATE MOUNT') + ' seconds on the board=' + (count '^SKATE (Ground|Air|Grind|Bail) speed') + ' top speed=' + (most ' top=([0-9.]+)' $false) + ' lowest cap=' + (most ' cap=([0-9.]+)' $true) + ' put backs=' + (count '^SKATE put back')
'tricks=' + (count '^SKATE trick ') + ' bails=' + (count '^SKATE bail') + ' grinds=' + (count 'Grind speed') + ' threw=' + (count 'threw') + ' refused=' + (count 'refused')

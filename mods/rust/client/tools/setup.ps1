# One-time preparation of the staging folder. The generator build (../generator) left a .NET SDK and
# the corrected interop generator on this PC in folders named after their build date. This keeps a
# copy of each in the staging folder as sdk\ and generator\, so that the other scripts do not depend
# on those names. Run again after a new generator build; an existing copy is replaced only with
# $pagain set.
$dl = "$HOME\Downloads"; $probe = "$dl\claude-loader-probe"; $from = "$dl\codex-bep788-probe"
function Pick($filter, $marker) {
  $c = @(ls $from -Directory -Filter $filter -ErrorAction SilentlyContinue | ? { Test-Path (Join-Path $_.FullName $marker) } | sort Name)
  'candidates for ' + $filter + ': ' + (($c | % Name) -join ', ') | Write-Host
  if ($c.Count -gt 0) { $c[-1] }
}
function Size($d) { [int]((ls $d -Recurse -File -ErrorAction SilentlyContinue | measure Length -Sum).Sum / 1MB) }
$sdk = Pick 'interop-nested-build-*' 'source-build\sdk\dotnet.exe'
$gen = Pick 'interop-callee-prepared-*' 'apps\consumer\Check.runtimeconfig.json'
if (-not $sdk -or -not $gen) { 'ABORT: the generator build is not on this PC; see ../generator/README.md'; return }
$jobs = @(@{ Name = 'sdk'; From = (Join-Path $sdk.FullName 'source-build\sdk'); Mark = 'dotnet.exe' }, @{ Name = 'generator'; From = $gen.FullName; Mark = 'apps\consumer\Check.runtimeconfig.json' })
$free = [int]((Get-PSDrive C).Free / 1MB)
foreach ($j in $jobs) {
  $to = Join-Path $probe $j.Name
  if ((Test-Path (Join-Path $to $j.Mark)) -and -not $pagain) { $j.Name + ': already there (' + (gc "$to\origin.txt" -First 1 -ErrorAction SilentlyContinue) + ')'; continue }
  $mb = Size $j.From
  if ($free -lt $mb * 2 + 2048) { 'ABORT: ' + $mb + ' MB to copy for ' + $j.Name + ' and only ' + $free + ' MB free'; return }
  if (Test-Path $to) { Rename-Item $to ($j.Name + '-before-' + (Get-Date -Format 'HHmmss')) }
  # A few tool files deep inside the SDK have paths too long to copy; the compiler is not among them.
  $skipped = @()
  Copy-Item $j.From $to -Recurse -ErrorAction SilentlyContinue -ErrorVariable skipped
  ('copied from ' + $j.From.Replace($dl, '~') + ' on ' + (Get-Date -Format 's')) | Out-File "$to\origin.txt" -Encoding utf8
  $j.Name + ': ' + (Size $to) + ' MB copied from ' + $j.From.Replace($dl, '~') + $(if ($skipped) { ', ' + @($skipped).Count + ' files left out (path too long)' })
}
'sdk compiler present=' + [bool](ls "$probe\sdk\sdk" -Directory -ErrorAction SilentlyContinue | ? { Test-Path "$($_.FullName)\Roslyn\bincore\csc.dll" }) + ' generator host present=' + (Test-Path "$probe\generator\dotnet-host\dotnet.exe") + ' free MB=' + [int]((Get-PSDrive C).Free / 1MB)

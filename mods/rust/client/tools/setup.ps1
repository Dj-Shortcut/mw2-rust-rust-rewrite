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
# An interop set made before the base libraries were kept per Unity version carries its own copy of
# them. The set for the installed build was made for this client, so its copy is of this version.
$rust = 'C:\Program Files (x86)\Steam\steamapps\common\Rust'
$pb = ((gc 'C:\Program Files (x86)\Steam\steamapps\appmanifest_252490.acf' | sls '"buildid"').Line -split '"')[3]
$uv = ''; if ("" + (gi "$rust\UnityPlayer.dll").VersionInfo.ProductVersion -match '^(\d+\.\d+\.\d+)') { $uv = $Matches[1] }
if ($uv -and -not (Test-Path "$probe\unity-libs-$uv\UnityEngine.CoreModule.dll") -and (Test-Path "$probe\gen-$pb\unity-libs\UnityEngine.CoreModule.dll")) {
  Copy-Item "$probe\gen-$pb\unity-libs" "$probe\unity-libs-$uv" -Recurse
  "unity base libraries for $uv taken from the set of build $pb"
}
'unity ' + $uv + ' base libraries present=' + (Test-Path "$probe\unity-libs-$uv\UnityEngine.CoreModule.dll")
'sdk compiler present=' + [bool](ls "$probe\sdk\sdk" -Directory -ErrorAction SilentlyContinue | ? { Test-Path "$($_.FullName)\Roslyn\bincore\csc.dll" }) + ' generator host present=' + (Test-Path "$probe\generator\dotnet-host\dotnet.exe") + ' free MB=' + [int]((Get-PSDrive C).Free / 1MB)

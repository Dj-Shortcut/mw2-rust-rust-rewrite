$cx = "$HOME\Downloads\codex-bep788-probe"; $prep = "$cx\interop-callee-prepared-20261009-007748-v1"
'apps: ' + ((ls "$prep\apps" -Recurse -File | % { $_.FullName.Replace("$prep\apps\",'') }) -join ', ')
gc "$prep\apps\consumer\Check.runtimeconfig.json" | % { $_.Trim() } | ? { $_ } | % { $_ } | Out-String | % { $_ -replace "\r?\n",' ' }
'--- no-arg run'
$out = & "$prep\dotnet-host\dotnet.exe" "$prep\apps\consumer\Check.dll" 2>&1
"exit=$LASTEXITCODE"
$out | select -First 12 | % { $s="$_"; if ($s.Length -gt 600) { $s.Substring(0,600) } else { $s } }
'--- strings'
$b = [IO.File]::ReadAllBytes("$prep\apps\consumer\Check.dll")
$t = [Text.Encoding]::Unicode.GetString($b)
$m = [regex]::Matches($t, '[\x20-\x7E]{6,200}') | % Value | ? { $_ -match 'arg|usage|path|dir|--|\.dll|\.dat|unity|output|input|expected' } | select -Unique
'count=' + $m.Count
($m | select -First 70) -join ' ¦ '

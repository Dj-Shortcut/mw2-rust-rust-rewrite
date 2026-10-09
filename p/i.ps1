ls $plast -Recurse -File -Depth 1 | ? { $_.DirectoryName -notmatch 'removed\\(BepInEx|dotnet)' } | % { "{0} {1}" -f $_.FullName.Replace($plast,'.'), $_.Length }
$lg = "$plast\removed\BepInEx\LogOutput.log"
"removed log exists=" + (Test-Path $lg)
foreach ($f in "$plast\LogOutput.log", $lg) { if ((Test-Path $f) -and (gi $f).Length -gt 0) { gc $f | ? { $_ -match 'PROBE|STEP|Error|Fatal' } | % { if ($_.Length -gt 1900) { $_.Substring(0,1900) } else { $_ } }; break } }

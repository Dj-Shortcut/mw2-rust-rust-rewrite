$rust = 'C:\Program Files (x86)\Steam\steamapps\common\Rust'; $dl = "$HOME\Downloads"
$acf = gc 'C:\Program Files (x86)\Steam\steamapps\appmanifest_252490.acf'
'acf: ' + (($acf | sls '"buildid"|"LastUpdated"|"StateFlags"|"TargetBuildID"' | % { $_.Line.Trim() -replace '\s+',' ' }) -join ' ; ')
'lastupdated: ' + [DateTimeOffset]::FromUnixTimeSeconds([long](($acf | sls '"LastUpdated"').Line -split '"')[3]).LocalDateTime
$md = "$rust\RustClient_Data\il2cpp_data\Metadata\global-metadata.dat"
foreach ($f in "$rust\GameAssembly.dll", $md, "$rust\UnityPlayer.dll") { $i = gi $f; "{0} {1} {2:yyyy-MM-dd HH:mm} sha={3}" -f $i.Name, $i.Length, $i.LastWriteTime, (Get-FileHash $f).Hash.Substring(0,16) }
'--- copies under Downloads'
ls $dl -Recurse -Depth 7 -Include 'global-metadata.dat','GameAssembly.dll' -ErrorAction SilentlyContinue | % { "{0} {1} {2:MM-dd HH:mm} sha={3} {4}" -f $_.Name, $_.Length, $_.LastWriteTime, (Get-FileHash $_.FullName).Hash.Substring(0,16), $_.DirectoryName.Replace($dl,'~') }
'--- generation dir'
$g = "$dl\codex-bep788-probe\interop-callee-generated-20261009-007748-v1"
ls $g -Depth 1 | select -First 30 | % { "{0} {1}" -f $_.FullName.Replace($g,'.'), $_.Length }

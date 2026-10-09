$dl = "$HOME\Downloads"; $cx = "$dl\codex-bep788-probe"
$prep = "$cx\interop-callee-prepared-20261009-007748-v1"; $g = "$cx\interop-callee-generated-20261009-007748-v1"
'--- prepared'
ls $prep | % { "{0} {1}" -f $_.Name, $(if ($_.PSIsContainer) { 'dir n=' + (ls $_.FullName -Recurse -File).Count } else { $_.Length }) }
ls $prep -Recurse -Depth 2 -Include *.ps1,*.exe,*.runtimeconfig.json,*.cmd,*.json -File | ? { $_.DirectoryName -notmatch 'dotnet-host\\(shared|host)' } | select -First 30 | % { $_.FullName.Replace($prep,'.') + ' ' + $_.Length }
'--- observation keys'
$o = gc "$g\generation-observation.json" -Raw
$j = $o | ConvertFrom-Json
($j | gm -MemberType NoteProperty | % Name) -join ', '
foreach ($k in 'command','commandLine','arguments','args','childProcesses','child','process','invocation') { if ($j.$k) { "$k = " + (($j.$k | ConvertTo-Json -Depth 4 -Compress)) } }
[regex]::Matches($o, '"[A-Za-z]*(Path|Dir|Root|Arguments|CommandLine|FileName)[A-Za-z]*":"[^"]{0,220}"') | select -First 40 | % { $_.Value }

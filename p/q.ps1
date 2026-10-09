$l = gc "$plast\LogOutput.log"
'lines=' + $l.Count
$l | ? { $_ -notmatch 'DobbyDetour|NativeDetour|Preloader\]|Cpp2IL' } | select -Last 34 | % { if ($_.Length -gt 330) { $_.Substring(0,330) } else { $_ } }

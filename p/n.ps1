# Generation status.
$g = $pgen; if (-not $g) { $g = (ls "$HOME\Downloads\claude-loader-probe" -Directory -Filter 'gen-*' | sort LastWriteTime | select -Last 1).FullName; $global:pgen = $g }
$pr = Get-Process -Name dotnet -ErrorAction SilentlyContinue | ? { $_.Path -like '*interop-callee-prepared*' }
'running=' + [bool]$pr + $(if ($pr) { ' mem=' + [int]($pr.WorkingSet64/1MB) + 'MB cpu=' + [int]$pr.CPU + 's' } else { '' }) + ' out files=' + $(if (Test-Path "$g\out") { (ls "$g\out").Count } else { 0 }) + ' free=' + [int]((Get-CimInstance Win32_OperatingSystem).FreePhysicalMemory/1MB) + 'GB'
gc "$g\gen.log" -ErrorAction SilentlyContinue | select -Last 16 | % { if ($_.Length -gt 420) { $_.Substring(0,420) } else { $_ } }
$e = gc "$g\gen.err" -ErrorAction SilentlyContinue; if ($e) { '--- stderr'; $e | select -Last 8 | % { if ($_.Length -gt 300) { $_.Substring(0,300) } else { $_ } } }

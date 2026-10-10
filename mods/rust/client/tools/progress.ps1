# Progress of the interop generation started by `zz interop`.
$g = $pgen; if (-not $g) { $g = (ls "$HOME\Downloads\claude-loader-probe" -Directory -Filter 'gen-*' | sort LastWriteTime | select -Last 1).FullName; $global:pgen = $g }
$pr = Get-Process -Name dotnet -ErrorAction SilentlyContinue | ? { $_.Path -like '*claude-loader-probe\generator\*' }
'running=' + [bool]$pr + $(if ($pr) { ' mem=' + [int]($pr.WorkingSet64/1MB) + 'MB cpu=' + [int]$pr.CPU + 's' } else { '' }) + ' out files=' + $(if (Test-Path "$g\out") { (ls "$g\out").Count } else { 0 }) + ' free=' + [int]((Get-CimInstance Win32_OperatingSystem).FreePhysicalMemory/1MB) + 'GB'
gc "$g\gen.log" -ErrorAction SilentlyContinue | select -Last 16 | % { if ($_.Length -gt 420) { $_.Substring(0,420) } else { $_ } }
$e = gc "$g\gen.err" -ErrorAction SilentlyContinue; if ($e) { '--- stderr'; $e | select -Last 8 | % { if ($_.Length -gt 300) { $_.Substring(0,300) } else { $_ } } }
# A trial set ($pgentest) is compared with the set in use for the same build.
if ($g -like '*\gentest-*' -and (Test-Path "$g\out") -and -not $pr) {
  $real = $g -replace 'gentest-', 'gen-'
  if (Test-Path "$real\out") {
    $a = @{}; ls "$real\out" -File | % { $a[$_.Name] = $_ }
    $b = @{}; ls "$g\out" -File | % { $b[$_.Name] = $_ }
    $missing = @($a.Keys | ? { -not $b.ContainsKey($_) }); $extra = @($b.Keys | ? { -not $a.ContainsKey($_) })
    $both = @($a.Keys | ? { $b.ContainsKey($_) })
    $size = @($both | ? { $a[$_].Length -ne $b[$_].Length })
    $same = @($both | ? { (Get-FileHash $a[$_].FullName).Hash -eq (Get-FileHash $b[$_].FullName).Hash })
    'compared with ' + (Split-Path $real -Leaf) + ': ' + $a.Count + ' files there, ' + $b.Count + ' here; missing ' + $missing.Count + ', extra ' + $extra.Count + ', other size ' + $size.Count + ', identical content ' + $same.Count + $(if ($size) { ' | other size: ' + (($size | select -First 6) -join ', ') })
  }
}

# Client-side readiness: Steam, installed Rust build, matching interop set, leftovers.
if (-not (Get-Process steam -ErrorAction SilentlyContinue)) {
  Start-Process 'C:\Program Files (x86)\Steam\steam.exe' -ArgumentList '-silent'
  'steam was not running; started it'; Start-Sleep 30
}
'steam running=' + [bool](Get-Process steam -ErrorAction SilentlyContinue) + ' webhelpers=' + @(Get-Process steamwebhelper -ErrorAction SilentlyContinue).Count
$acf = gc 'C:\Program Files (x86)\Steam\steamapps\appmanifest_252490.acf'
$val = { param($k) (($acf | sls ('"' + $k + '"') | select -First 1).Line -split '"')[3] }
$pb = & $val 'buildid'
'rust build=' + $pb + ' stateflags=' + (& $val 'StateFlags') + ' target=' + (& $val 'TargetBuildID')
$probe = "$HOME\Downloads\claude-loader-probe"
'interop for this build=' + (Test-Path "$probe\gen-$pb\out\Assembly-CSharp.dll") + ' | sets: ' + ((ls $probe -Directory -Filter 'gen-*' | % Name) -join ',')
$rust = 'C:\Program Files (x86)\Steam\steamapps\common\Rust'
'bootstrap roots in rust dir: ' + (('BepInEx','dotnet','winhttp.dll','doorstop_config.ini','.doorstop_version','changelog.txt' | ? { Test-Path (Join-Path $rust $_) }) -join ',')
'rust running=' + [bool](Get-Process RustClient -ErrorAction SilentlyContinue) + ' free GB=' + [int]((Get-CimInstance Win32_OperatingSystem).FreePhysicalMemory/1MB) + ' total GB=' + [int]((Get-CimInstance Win32_ComputerSystem).TotalPhysicalMemory/1GB)
'patched runtime present=' + (Test-Path "$probe\rt\Il2CppInterop.Runtime.dll") + ' plugin present=' + (Test-Path "$probe\plugin\LoaderProbe.dll")

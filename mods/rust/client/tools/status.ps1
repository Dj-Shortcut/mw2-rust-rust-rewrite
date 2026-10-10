# Readiness of this PC for a session: Steam, the installed Rust build, the interop set for that
# build, leftovers in the Rust folder, the patched runtime, the built plugin and the launcher.
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
'interop for this build=' + ((Test-Path "$probe\gen-$pb\out\Assembly-CSharp.dll") -and [bool](sls -Path "$probe\gen-$pb\gen.log" -Pattern 'GEN DONE.* errors=0( |$)' -Quiet -ErrorAction SilentlyContinue)) + ' | sets: ' + ((ls $probe -Directory -Filter 'gen-*' | % Name) -join ',')
'last line of its log: ' + $(if (Test-Path "$probe\gen-$pb\gen.log") { $t = "" + (gc "$probe\gen-$pb\gen.log" -Tail 1); if ($t.Length -gt 160) { $t.Substring(0, 160) } else { $t } } else { 'no log' })
$rust = 'C:\Program Files (x86)\Steam\steamapps\common\Rust'
'loader files in the rust folder: ' + (('BepInEx','dotnet','winhttp.dll','doorstop_config.ini','.doorstop_version','changelog.txt' | ? { Test-Path -LiteralPath (Join-Path $rust $_) }) -join ',') + ' (empty means clean)'
'rust running=' + [bool](Get-Process RustClient -ErrorAction SilentlyContinue) + ' free GB=' + [int]((Get-CimInstance Win32_OperatingSystem).FreePhysicalMemory/1MB) + ' total GB=' + [int]((Get-CimInstance Win32_ComputerSystem).TotalPhysicalMemory/1GB)
'patched runtime present=' + (Test-Path "$probe\rt\Il2CppInterop.Runtime.dll") + ' plugin built=' + (Test-Path "$probe\plugin\ShortcutSkateClient.dll")
'launcher: ' + $(if (Test-Path "$probe\launcher\version.txt") { gc "$probe\launcher\version.txt" -First 1 } else { 'not installed' })

# Transport helper + read-only inventory for the client-loader probe (issue #289).
function global:r($n) { iex (irm ("https://raw.githubusercontent.com/Dj-Shortcut/mw2-rust-rust-rewrite/claude-loader-probe/p/{0}.ps1" -f $n)) }
$ErrorActionPreference = 'SilentlyContinue'
$o = @()
$o += "ps=" + $PSVersionTable.PSVersion + " git=" + [bool](gcm git) + " dotnet=" + [bool](gcm dotnet) + " gh=" + [bool](gcm gh) + " 7z=" + [bool](gcm 7z)
if (gcm dotnet) { $o += "sdks: " + ((dotnet --list-sdks) -join ' | ') }
$libs = @("C:\Program Files (x86)\Steam") + (gc "C:\Program Files (x86)\Steam\steamapps\libraryfolders.vdf" | sls '"path"' | % { ($_ -split '"')[3] -replace '\\\\','\' })
$rust = $libs | % { Join-Path $_ 'steamapps\common\Rust' } | ? { Test-Path (Join-Path $_ 'RustClient.exe') } | select -First 1
$o += "rust=$rust"
if ($rust) {
  $o += "rustroot: " + ((ls $rust | % Name) -join ', ')
  $o += "unity: " + (gi (Join-Path $rust 'UnityPlayer.dll')).VersionInfo.ProductVersion
}
$dl = "$HOME\Downloads"
$o += "downloads dirs=" + (ls $dl -Directory).Count + " files=" + (ls $dl -File).Count
$o += "free GB=" + [math]::Round((gdr C).Free/1GB,1)
# directories that hold a generated interop set (many dlls incl. Assembly-CSharp.dll)
$sets = ls $dl -Recurse -Filter 'Assembly-CSharp.dll' -Depth 6 | % { $d=$_.Directory; "{0:MM-dd HH:mm} n={1} {2}" -f $_.LastWriteTime,(ls $d.FullName -Filter *.dll).Count,$d.FullName.Replace($dl,'~') }
$o += "interop sets:"; $o += ($sets | sort | select -Last 12)
$o += "bepinex zips: " + ((ls $dl -Recurse -Depth 3 -Include 'BepInEx*.zip' | % { $_.FullName.Replace($dl,'~') }) -join ' ; ')
$o | Out-File "$HOME\claude-probe-a.txt" -Width 400
$o | % { $_ }

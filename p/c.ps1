$dl = "$HOME\Downloads"
ls $dl -Recurse -Depth 5 -Filter dotnet.exe -ErrorAction SilentlyContinue | % {
  $d = $_.Directory.FullName
  $sdk = if (Test-Path "$d\sdk") { (ls "$d\sdk" -Directory | % Name) -join ',' } else { '-' }
  $rt = if (Test-Path "$d\shared\Microsoft.NETCore.App") { (ls "$d\shared\Microsoft.NETCore.App" -Directory | % Name) -join ',' } else { '-' }
  "{0} | sdk={1} | rt={2}" -f $d.Replace($dl,'~'), $sdk, $rt
}
'pwsh: ' + ((ls $dl -Recurse -Depth 4 -Filter pwsh.exe -ErrorAction SilentlyContinue | % { $_.FullName.Replace($dl,'~') }) -join ' ; ')
'nuget caches: ' + (Test-Path "$HOME\.nuget\packages") + ' n=' + (ls "$HOME\.nuget\packages" -ErrorAction SilentlyContinue).Count
'csc fw: ' + (Test-Path 'C:\Windows\Microsoft.NET\Framework64\v4.0.30319\csc.exe')
'core dlls: ' + ((ls "$dl\claude-loader-probe\bep788\BepInEx\core" | % Name) -join ', ')

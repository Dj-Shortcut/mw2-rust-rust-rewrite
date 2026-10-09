# Compile the loader probe plugin on Shadow with the staged SDK's Roslyn compiler.
$dl = "$HOME\Downloads"; $probe = "$dl\claude-loader-probe"
$sdk = "$dl\codex-bep788-probe\interop-nested-build-20261007-c5ab5f\source-build\sdk"
$gen = "$dl\codex-bep788-probe\interop-callee-generated-20261009-007748-v1\generation\generated"
# Prefer an interop set generated for the installed build, when one exists.
$pb = ((gc 'C:\Program Files (x86)\Steam\steamapps\appmanifest_252490.acf' | sls '"buildid"').Line -split '"')[3]
if (Test-Path "$dl\claude-loader-probe\gen-$pb\out\Assembly-CSharp.dll") { $gen = "$dl\claude-loader-probe\gen-$pb\out" }
$core = "$probe\bep788\BepInEx\core"; $fx = "$probe\bep788\dotnet"
New-Item -ItemType Directory -Force "$probe\plugin","$probe\src" | Out-Null
$src = "$probe\src\Probe.cs"
(zg Probe.cs) | Out-File $src -Encoding utf8
$refs = @()
$refs += ls $fx -Filter *.dll | ? { ($_.Name -like 'System*' -or $_.Name -in 'mscorlib.dll','netstandard.dll','Microsoft.CSharp.dll') -and $_.Name -notlike '*.Native.dll' } | % FullName
$refs += 'BepInEx.Core.dll','BepInEx.Unity.IL2CPP.dll','BepInEx.Unity.Common.dll','Il2CppInterop.Runtime.dll','Il2CppInterop.Common.dll' | % { "$core\$_" }
$refs += 'UnityEngine.CoreModule.dll','Il2Cppmscorlib.dll','Il2CppSystem.dll','Il2CppSystem.Core.dll' | % { "$gen\$_" }
$rsp = "$probe\src\probe.rsp"
(@('-nologo','-target:library','-nostdlib','-optimize+','-nowarn:CS1701,CS1702,CS8632',"-out:`"$probe\plugin\LoaderProbe.dll`"") + ($refs | % { "-r:`"$_`"" }) + "`"$src`"") | Out-File $rsp -Encoding ascii
$csc = (ls "$sdk\sdk" -Directory | select -First 1).FullName + '\Roslyn\bincore\csc.dll'
$out = & "$sdk\dotnet.exe" $csc "@$rsp" 2>&1
"csc exit=$LASTEXITCODE refs=" + $refs.Count
$out | select -First 25 | % { $s = "$_"; if ($s.Length -gt 260) { $s.Substring(0,260) } else { $s } }
ls "$probe\plugin" | % { "{0} {1}" -f $_.Name, $_.Length }

if ($LASTEXITCODE -eq 0 -and $pnext) { zz $pnext }

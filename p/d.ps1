# Compile the loader probe plugin on Shadow with the staged SDK's Roslyn compiler.
$dl = "$HOME\Downloads"; $probe = "$dl\claude-loader-probe"
$sdk = "$dl\codex-bep788-probe\interop-nested-build-20261007-c5ab5f\source-build\sdk"
# The interop set must have been generated for the installed Steam build. A set from another build
# resolves wrappers to the wrong native methods and crashes (issue #289, 9 October 2026), so there
# is no fallback: generate first with `zz m`.
$pb = ((gc 'C:\Program Files (x86)\Steam\steamapps\appmanifest_252490.acf' | sls '"buildid"').Line -split '"')[3]
$gen = "$dl\claude-loader-probe\gen-$pb\out"
if (-not ((Test-Path "$gen\Assembly-CSharp.dll") -and (sls -Path "$dl\claude-loader-probe\gen-$pb\gen.log" -Pattern 'GEN DONE' -Quiet))) { Write-Host "ABORT: no completed interop set for installed build $pb; run zz m first"; return }
$core = "$probe\bep788\BepInEx\core"; $fx = "$probe\bep788\dotnet"
New-Item -ItemType Directory -Force "$probe\plugin","$probe\src" | Out-Null
$src = "$probe\src\Probe.cs"
(zg Probe.cs) | Out-File $src -Encoding utf8
$refs = @()
$refs += ls $fx -Filter *.dll | ? { ($_.Name -like 'System*' -or $_.Name -in 'mscorlib.dll','netstandard.dll','Microsoft.CSharp.dll') -and $_.Name -notlike '*.Native.dll' } | % FullName
$refs += 'BepInEx.Core.dll','BepInEx.Unity.IL2CPP.dll','BepInEx.Unity.Common.dll','Il2CppInterop.Runtime.dll','Il2CppInterop.Common.dll' | % { "$core\$_" }
# All generated interop assemblies, so game and engine types resolve without a hand-kept list.
$refs += ls $gen -Filter *.dll | % FullName
# Shared sources from another branch, taken at a pinned commit and checked against a SHA-256.
$shared = @(@{ Path = 'mods/rust/shared/SkateBoardMesh.cs'; Commit = '68ddb15c0090ce880693942823d40e8d69fa8ea2'; Sha = '6391708873a9da2e3b295edee8aff9507d8d0780460af5b1eeaa71d503dbffdc' })
$extra = @()
foreach ($f in $shared) {
  $dst = "$probe\src\" + (Split-Path $f.Path -Leaf)
  Invoke-WebRequest -Uri ('https://raw.githubusercontent.com/Dj-Shortcut/mw2-rust-rust-rewrite/' + $f.Commit + '/' + $f.Path) -OutFile $dst -UseBasicParsing
  if ((Get-FileHash $dst -Algorithm SHA256).Hash.ToLower() -ne $f.Sha) { Write-Host ("ABORT: " + $f.Path + " does not match its pinned hash"); return }
  $extra += $dst
}
$rsp = "$probe\src\probe.rsp"
(@('-nologo','-target:library','-nostdlib','-optimize+','-nowarn:CS1701,CS1702,CS8632',"-out:`"$probe\plugin\LoaderProbe.dll`"") + ($refs | % { "-r:`"$_`"" }) + "`"$src`"" + ($extra | % { "`"$_`"" })) | Out-File $rsp -Encoding ascii
$csc = (ls "$sdk\sdk" -Directory | select -First 1).FullName + '\Roslyn\bincore\csc.dll'
$out = & "$sdk\dotnet.exe" $csc "@$rsp" 2>&1
"csc exit=$LASTEXITCODE refs=" + $refs.Count
$out | select -First 25 | % { $s = "$_"; if ($s.Length -gt 260) { $s.Substring(0,260) } else { $s } }
ls "$probe\plugin" | % { "{0} {1}" -f $_.Name, $_.Length }

if ($LASTEXITCODE -eq 0 -and $pnext) { zz $pnext }

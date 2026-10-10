# Compile the skate client plugin on Shadow with the staged SDK's Roslyn compiler.
# Sources are listed in mods/rust/client/sources.txt on this branch.
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
$repo = 'Dj-Shortcut/mw2-rust-rust-rewrite'
# One API request for the head of this branch; every file then comes from the raw host at that
# exact commit, which cannot be stale. Lines with their own commit and SHA-256 are pinned shared code.
# $pcommit builds that exact commit instead (the launcher's released build).
$head = "$pcommit"
if (-not $head) { $head = ("" + (irm -Headers @{ Accept = 'application/vnd.github.sha' } "https://api.github.com/repos/$repo/commits/claude-loader-probe")).Trim() }
if ($head -notmatch '^[0-9a-f]{40}$') { Write-Host 'ABORT: could not read the commit to build'; return }
$global:pbuilt = $null
$list = ("" + (irm "https://raw.githubusercontent.com/$repo/$head/mods/rust/client/sources.txt")) -split "`n"
Remove-Item "$probe\src\*.cs" -ErrorAction SilentlyContinue
$files = @()
foreach ($line in $list) {
  $t = $line.Trim(); if (-not $t -or $t.StartsWith('#')) { continue }
  $parts = $t -split '\s+'
  $commit = $head; if ($parts.Count -ge 3) { $commit = $parts[1] }
  $dst = "$probe\src\" + (Split-Path $parts[0] -Leaf)
  Invoke-WebRequest -Uri ("https://raw.githubusercontent.com/$repo/$commit/" + $parts[0]) -OutFile $dst -UseBasicParsing
  if ($parts.Count -ge 3 -and (Get-FileHash $dst -Algorithm SHA256).Hash.ToLower() -ne $parts[2]) { Write-Host ("ABORT: " + $parts[0] + " does not match its pinned hash"); return }
  $files += $dst
}
$refs = @()
$refs += ls $fx -Filter *.dll | ? { ($_.Name -like 'System*' -or $_.Name -in 'mscorlib.dll','netstandard.dll','Microsoft.CSharp.dll') -and $_.Name -notlike '*.Native.dll' } | % FullName
$refs += 'BepInEx.Core.dll','BepInEx.Unity.IL2CPP.dll','BepInEx.Unity.Common.dll','Il2CppInterop.Runtime.dll','Il2CppInterop.Common.dll' | % { "$core\$_" }
# All generated interop assemblies, so game and engine types resolve without a hand-kept list.
$refs += ls $gen -Filter *.dll | % FullName
# Exactly one plugin may be installed: an older build under another name would run as well.
Remove-Item "$probe\plugin\*.dll" -ErrorAction SilentlyContinue
$rsp = "$probe\src\client.rsp"
(@('-nologo','-target:library','-nostdlib','-optimize+','-nowarn:CS1701,CS1702,CS8632',"-out:`"$probe\plugin\ShortcutSkateClient.dll`"") + ($refs | % { "-r:`"$_`"" }) + ($files | % { "`"$_`"" })) | Out-File $rsp -Encoding ascii
$csc = (ls "$sdk\sdk" -Directory | select -First 1).FullName + '\Roslyn\bincore\csc.dll'
$out = & "$sdk\dotnet.exe" $csc "@$rsp" 2>&1
"csc exit=$LASTEXITCODE sources=" + $files.Count + " refs=" + $refs.Count + " head=" + $head.Substring(0, 7)
$out | select -First 25 | % { $s = "$_"; if ($s.Length -gt 260) { $s.Substring(0,260) } else { $s } }
ls "$probe\plugin" | % { "{0} {1}" -f $_.Name, $_.Length }

if ($LASTEXITCODE -eq 0) { $global:pbuilt = $head }
if ($LASTEXITCODE -eq 0 -and $pnext) { zz $pnext }

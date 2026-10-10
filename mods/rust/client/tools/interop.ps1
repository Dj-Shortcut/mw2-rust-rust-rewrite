# Generate the interop assemblies for the installed Rust build (needed again after every Rust
# update): compiles Gen.cs against the prepared generator and starts it in the background.
# Follow it with `zz interop-status`; the set is complete when its log ends with GEN DONE.
$dl = "$HOME\Downloads"; $probe = "$dl\claude-loader-probe"; $cx = "$dl\codex-bep788-probe"
$prep = "$cx\interop-callee-prepared-20261009-007748-v1"
$sdk = "$cx\interop-nested-build-20261007-c5ab5f\source-build\sdk"
$rust = 'C:\Program Files (x86)\Steam\steamapps\common\Rust'
$acf = gc 'C:\Program Files (x86)\Steam\steamapps\appmanifest_252490.acf'
$build = (($acf | sls '"buildid"').Line -split '"')[3]
$gen = "$probe\gen-$build"
if (Get-Process -Name dotnet -ErrorAction SilentlyContinue | ? { $_.Path -like "$prep*" }) { 'ABORT: a generation process is already running'; return }
if (Test-Path "$gen\out") { 'ABORT: output already exists: ' + "$gen\out"; return }
New-Item -ItemType Directory -Force "$gen\app" | Out-Null
Copy-Item "$prep\apps\consumer\*" "$gen\app" -Exclude 'Check.*'
Copy-Item "$prep\apps\consumer\Check.runtimeconfig.json" "$gen\app\Gen.runtimeconfig.json"
# Unity base libraries: reuse an extracted set on disk, else download the official archive.
$ul = "$gen\unity-libs"
$found = ls $cx -Recurse -Depth 6 -Filter 'UnityEngine.CoreModule.dll' -ErrorAction SilentlyContinue | ? { $_.DirectoryName -notmatch 'generated' -and (ls $_.DirectoryName -Filter *.dll).Count -lt 120 -and (ls $_.DirectoryName -Filter *.dll).Count -gt 60 } | select -First 1
if ($found) { Copy-Item $found.DirectoryName $ul -Recurse; 'unity libs from ' + $found.DirectoryName.Replace($dl,'~') }
else {
  New-Item -ItemType Directory -Force $ul | Out-Null
  Invoke-WebRequest 'https://unity.bepinex.dev/libraries/6000.3.15.zip' -OutFile "$gen\unity-libs.zip" -UseBasicParsing
  Expand-Archive "$gen\unity-libs.zip" $ul
  'unity libs downloaded'
}
'unity libs n=' + (ls $ul -Filter *.dll).Count
(zg Gen.cs) | Out-File "$gen\Gen.cs" -Encoding utf8
$fx = "$probe\bep788\dotnet"
$refs = @(ls $fx -Filter *.dll | ? { ($_.Name -like 'System*' -or $_.Name -in 'mscorlib.dll','netstandard.dll','Microsoft.CSharp.dll') -and $_.Name -notlike '*.Native.dll' } | % FullName)
$refs += ls "$gen\app" -Filter *.dll | % FullName
(@('-nologo','-target:exe','-nostdlib','-optimize+','-nowarn:CS1701,CS1702,CS8632,CS8625',"-out:`"$gen\app\Gen.dll`"") + ($refs | % { "-r:`"$_`"" }) + "`"$gen\Gen.cs`"") | Out-File "$gen\gen.rsp" -Encoding ascii
$csc = (ls "$sdk\sdk" -Directory | select -First 1).FullName + '\Roslyn\bincore\csc.dll'
$o = & "$sdk\dotnet.exe" $csc "@$gen\gen.rsp" 2>&1
"csc exit=$LASTEXITCODE"
$o | select -First 14 | % { $s = "$_"; if ($s.Length -gt 300) { $s.Substring(0,300) } else { $s } }
if ($LASTEXITCODE -ne 0) { return }
$md = "$rust\RustClient_Data\il2cpp_data\Metadata\global-metadata.dat"
$al = "`"$gen\app\Gen.dll`" `"$rust\GameAssembly.dll`" `"$md`" `"$ul`" `"$gen\out`""
$p = Start-Process -FilePath "$prep\dotnet-host\dotnet.exe" -ArgumentList $al -WorkingDirectory "$gen\app" -RedirectStandardOutput "$gen\gen.log" -RedirectStandardError "$gen\gen.err" -WindowStyle Hidden -PassThru
$global:pgen = $gen
"started pid=" + $p.Id + " build=$build dir=" + $gen.Replace($dl,'~')

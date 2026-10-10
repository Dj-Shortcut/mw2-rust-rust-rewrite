# Generate the interop assemblies for the installed Rust build (needed again after every Rust
# update): compiles Gen.cs against the prepared generator and starts it in the background.
# Follow it with `zz progress`; the set is complete when its log ends with GEN DONE.
# With $pgentest the set goes to a folder of its own (gentest-<build>), to try the generation
# without replacing a set that is in use.
$dl = "$HOME\Downloads"; $probe = "$dl\claude-loader-probe"
$prep = "$probe\generator"; $sdk = "$probe\sdk"
if (-not ((Test-Path "$sdk\dotnet.exe") -and (Test-Path "$prep\dotnet-host\dotnet.exe"))) { 'ABORT: no SDK or generator in the staging folder; run zz setup first'; return }
$rust = 'C:\Program Files (x86)\Steam\steamapps\common\Rust'
$acf = gc 'C:\Program Files (x86)\Steam\steamapps\appmanifest_252490.acf'
$build = (($acf | sls '"buildid"').Line -split '"')[3]
$gen = "$probe\gen-$build"; if ($pgentest) { $gen = "$probe\gentest-$build" }
# Cpp2IL and the base libraries must be told the client's own Unity version: the Steam build number
# says nothing about it, and a set made for another version resolves to the wrong native methods.
$uv = ''; if ("" + (gi "$rust\UnityPlayer.dll").VersionInfo.ProductVersion -match '^(\d+\.\d+\.\d+)') { $uv = $Matches[1] }
if (-not $uv) { 'ABORT: could not read the Unity version of the installed client'; return }
if (Get-Process -Name dotnet -ErrorAction SilentlyContinue | ? { $_.Path -like "$prep*" }) { 'ABORT: a generation process is already running'; return }
if (Test-Path "$gen\out") { 'ABORT: output already exists: ' + "$gen\out"; return }
# The official base libraries of that Unity version, fetched once and kept under its number.
$ul = "$probe\unity-libs-$uv"
if (-not (Test-Path "$ul\UnityEngine.CoreModule.dll")) {
  New-Item -ItemType Directory -Force $ul | Out-Null
  Invoke-WebRequest "https://unity.bepinex.dev/libraries/$uv.zip" -OutFile "$probe\unity-libs-$uv.zip" -UseBasicParsing
  Expand-Archive "$probe\unity-libs-$uv.zip" $ul -Force
  'unity base libraries downloaded'
}
if (-not (Test-Path "$ul\UnityEngine.CoreModule.dll")) { "ABORT: no Unity base libraries for $uv"; return }
"unity $uv, base libraries n=" + (ls $ul -Filter *.dll).Count
New-Item -ItemType Directory -Force "$gen\app" | Out-Null
Copy-Item "$prep\apps\consumer\*" "$gen\app" -Exclude 'Check.*'
Copy-Item "$prep\apps\consumer\Check.runtimeconfig.json" "$gen\app\Gen.runtimeconfig.json"
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
$al = "`"$gen\app\Gen.dll`" `"$rust\GameAssembly.dll`" `"$md`" `"$ul`" `"$gen\out`" $uv"
$p = Start-Process -FilePath "$prep\dotnet-host\dotnet.exe" -ArgumentList $al -WorkingDirectory "$gen\app" -RedirectStandardOutput "$gen\gen.log" -RedirectStandardError "$gen\gen.err" -WindowStyle Hidden -PassThru
$global:pgen = $gen
"started pid=" + $p.Id + " build=$build unity=$uv dir=" + $gen.Replace($dl,'~')

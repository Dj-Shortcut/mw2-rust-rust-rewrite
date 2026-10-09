# Build Il2CppInterop.Runtime 1.5.3 (pinned dbda1cb) with the two probe changes from p/rt.
$dl = "$HOME\Downloads"; $probe = "$dl\claude-loader-probe"; $cx = "$dl\codex-bep788-probe"
$sdk = "$cx\interop-nested-build-20261007-c5ab5f\source-build\sdk"
$commit = 'dbda1cb353b0f4253345dc45136d170b9e50a5a0'
$root = "$probe\rtsrc"; $src = "$root\Il2CppInterop-$commit"
if (-not (Test-Path "$src\Il2CppInterop.sln")) {
  New-Item -ItemType Directory -Force $root | Out-Null
  Invoke-WebRequest "https://codeload.github.com/BepInEx/Il2CppInterop/zip/$commit" -OutFile "$root\src.zip" -UseBasicParsing
  'source zip sha256=' + (Get-FileHash "$root\src.zip").Hash
  Expand-Archive "$root\src.zip" $root -Force
}
$enc = New-Object Text.UTF8Encoding($false)
[IO.File]::WriteAllText("$src\Il2CppInterop.Runtime\Injection\InjectorHelpers.cs", (zg 'rt/InjectorHelpers.cs'), $enc)
[IO.File]::WriteAllText("$src\Il2CppInterop.Runtime\Injection\Hooks\Class_GetFieldDefaultValue_Hook.cs", (zg 'rt/Class_GetFieldDefaultValue_Hook.cs'), $enc)
$env:DOTNET_CLI_HOME = "$probe\dotnet-home"; $env:NUGET_PACKAGES = "$probe\nuget"; $env:DOTNET_CLI_TELEMETRY_OPTOUT = '1'; $env:DOTNET_NOLOGO = '1'; $env:DOTNET_SKIP_FIRST_TIME_EXPERIENCE = '1'
$o = & "$sdk\dotnet.exe" build "$src\Il2CppInterop.Runtime\Il2CppInterop.Runtime.csproj" -c Release -p:GeneratePackageOnBuild=false -nologo -v q 2>&1
"build exit=$LASTEXITCODE"
$o | ? { "$_" -match 'error|Warn|Elapsed' } | select -First 14 | % { $s = "$_"; if ($s.Length -gt 300) { $s.Substring(0,300) } else { $s } }
$dll = ls "$src\bin\Il2CppInterop.Runtime" -Recurse -Filter Il2CppInterop.Runtime.dll -ErrorAction SilentlyContinue | sort LastWriteTime | select -Last 1
if ($LASTEXITCODE -eq 0 -and $dll) {
  New-Item -ItemType Directory -Force "$probe\rt" | Out-Null
  Copy-Item $dll.FullName "$probe\rt" -Force
  $stock = gi "$probe\bep788\BepInEx\core\Il2CppInterop.Runtime.dll"
  "built {0} bytes ver={1} | stock {2} bytes ver={3}" -f $dll.Length, $dll.VersionInfo.FileVersion, $stock.Length, $stock.VersionInfo.FileVersion
  if ($pnext) { $t = $pnext; $global:pnext = $null; zz $t }
}

# Project-authored build recipe. The pinned generator patch remains LGPL-3.0-only.
#Requires -Version 7.4
[CmdletBinding()]
param(
    [Parameter(Mandatory)][string]$Destination,
    [string]$DotNetPath,
    [string]$GitPath,
    [string]$SourceArchive
)
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$payload = $PSScriptRoot
$inputs = Get-Content -LiteralPath (Join-Path $payload 'build-inputs.json') -Raw | ConvertFrom-Json
function Assert-Hash([string]$Path, [string]$Expected, [string]$Algorithm = 'SHA256') {
    if ((Get-FileHash -LiteralPath $Path -Algorithm $Algorithm).Hash -ine $Expected) {
        throw "Hash mismatch: $Path"
    }
}
function Run([string]$Executable, [string[]]$Arguments) {
    & $Executable @Arguments
    if ($LASTEXITCODE -ne 0) { throw "Command failed ($LASTEXITCODE): $Executable" }
}
function Assert-Assembly([string]$Path, [string]$Name) {
    $identity = [Reflection.AssemblyName]::GetAssemblyName($Path)
    if ($identity.Name -cne $Name -or $identity.Version.ToString() -cne $inputs.assemblyVersion -or $identity.GetPublicKeyToken().Length -ne 0 -or $identity.CultureName) {
        throw "Unexpected assembly identity: $Name"
    }
    return $identity.FullName
}
$patch = Join-Path $payload 'pointer-byref.patch'
Assert-Hash $patch $inputs.patchSha256
$requiredLocks = @('Il2CppInterop.Generator.lock.json', 'Il2CppInterop.Common.lock.json')
if (@($inputs.lockFiles.PSObject.Properties).Count -ne $requiredLocks.Count) { throw 'The complete two-project restore lock set is required.' }
foreach ($name in $requiredLocks) {
    $entry = $inputs.lockFiles.PSObject.Properties[$name]
    if (!$entry) { throw "Missing restore lock: $name" }
    Assert-Hash (Join-Path $payload ('locks/' + $name)) $entry.Value
}
Assert-Hash (Join-Path $payload 'NuGet.Config') $inputs.nugetConfigSha256
foreach ($entry in $inputs.licenceFiles.PSObject.Properties) { Assert-Hash (Join-Path $payload $entry.Name) $entry.Value }
if (![IO.Path]::IsPathFullyQualified($Destination)) { throw 'Destination must be an absolute filesystem path.' }
$destinationPath = [IO.Path]::GetFullPath($Destination)
if (Test-Path -LiteralPath $destinationPath) { throw 'Destination must be a new directory; existing files are preserved.' }
$ancestor = [IO.DirectoryInfo]::new($destinationPath).Parent
while ($null -ne $ancestor) {
    $existingAncestor = $null
    try { $existingAncestor = Get-Item -LiteralPath $ancestor.FullName -Force -ErrorAction Stop }
    catch [System.Management.Automation.ItemNotFoundException] { }
    if ($null -ne $existingAncestor -and ($existingAncestor.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) {
        throw 'Destination must not pass through symbolic links or junctions.'
    }
    foreach ($name in @('Rust.exe', 'RustClient.exe', 'RustDedicated.exe', 'RustDedicated')) {
        if (Test-Path -LiteralPath (Join-Path $ancestor.FullName $name)) { throw 'Build outside the Rust game and server directories.' }
    }
    $ancestor = $ancestor.Parent
}
if ($SourceArchive) {
    $SourceArchive = (Resolve-Path -LiteralPath $SourceArchive).Path
    Assert-Hash $SourceArchive $inputs.sourceSha256
}
if (!$GitPath) {
    $gitCommand = Get-Command git -CommandType Application -ErrorAction SilentlyContinue
    if (!$gitCommand) { throw 'Supply an explicit path to Git or install Git before building.' }
    $GitPath = $gitCommand.Source
}
$GitPath = (Resolve-Path -LiteralPath $GitPath).Path
if ($DotNetPath) { $DotNetPath = (Resolve-Path -LiteralPath $DotNetPath).Path }
elseif (!$IsWindows) { throw "Supply an explicit path to SDK $($inputs.sdkVersion) dotnet on this platform." }
New-Item -ItemType Directory -Path $destinationPath | Out-Null
$transcript = Join-Path $destinationPath 'build-transcript.txt'
Start-Transcript -LiteralPath $transcript -NoClobber | Out-Null
$environmentNames = @('DOTNET_CLI_HOME', 'NUGET_PACKAGES', 'DOTNET_SKIP_FIRST_TIME_EXPERIENCE', 'DOTNET_CLI_TELEMETRY_OPTOUT', 'DOTNET_GENERATE_ASPNET_CERTIFICATE')
$sourceLocationPushed = $false
$priorEnvironment = @{}
foreach ($name in $environmentNames) { $priorEnvironment[$name] = [Environment]::GetEnvironmentVariable($name, 'Process') }
try {
    $env:DOTNET_CLI_HOME = Join-Path $destinationPath 'dotnet-home'
    $env:NUGET_PACKAGES = Join-Path $destinationPath 'packages'
    $env:DOTNET_SKIP_FIRST_TIME_EXPERIENCE = '1'
    $env:DOTNET_CLI_TELEMETRY_OPTOUT = '1'
    $env:DOTNET_GENERATE_ASPNET_CERTIFICATE = 'false'
    if (!$DotNetPath) {
        $sdkArchive = Join-Path $destinationPath 'sdk.zip'
        Invoke-WebRequest -Uri $inputs.windowsSdk.url -OutFile $sdkArchive
        Assert-Hash $sdkArchive $inputs.windowsSdk.hash 'SHA512'
        $sdkDirectory = Join-Path $destinationPath 'sdk'
        Expand-Archive -LiteralPath $sdkArchive -DestinationPath $sdkDirectory
        $DotNetPath = Join-Path $sdkDirectory 'dotnet.exe'
    }
    $installedSdk = @(& $DotNetPath --list-sdks) -match ('^' + [Regex]::Escape($inputs.sdkVersion) + ' \[')
    if ($LASTEXITCODE -ne 0 -or !$installedSdk) { throw "Required SDK $($inputs.sdkVersion) is not installed at $DotNetPath" }
    Run $GitPath @('--version')
    $archive = Join-Path $destinationPath 'source.zip'
    if ($SourceArchive) { Copy-Item -LiteralPath $SourceArchive -Destination $archive }
    else { Invoke-WebRequest -Uri $inputs.sourceUrl -OutFile $archive }
    Assert-Hash $archive $inputs.sourceSha256
    $unpacked = Join-Path $destinationPath 'upstream'
    Expand-Archive -LiteralPath $archive -DestinationPath $unpacked
    $source = Join-Path $unpacked ('Il2CppInterop-' + $inputs.sourceCommit)
    if (!(Test-Path -LiteralPath (Join-Path $source 'Il2CppInterop.Generator/Il2CppInterop.Generator.csproj'))) { throw 'Unexpected source archive layout.' }
    foreach ($entry in $inputs.preservedFiles.PSObject.Properties) { Assert-Hash (Join-Path $source $entry.Name) $entry.Value }
    Push-Location -LiteralPath $source
    $sourceLocationPushed = $true
    $sdkVersion = (& $DotNetPath --version | Out-String).Trim()
    if ($LASTEXITCODE -ne 0 -or $sdkVersion -cne $inputs.sdkVersion) { throw "Expected SDK $($inputs.sdkVersion); got $sdkVersion" }
    if (Test-Path -LiteralPath (Join-Path $source '.git')) { throw 'Source archive unexpectedly contains Git state.' }
    Run $GitPath @('-C', $source, 'init', '--quiet')
    Run $GitPath @('-c', 'core.autocrlf=false', '-c', 'core.eol=lf', '-C', $source, 'apply', '--check', '--whitespace=error-all', $patch)
    Run $GitPath @('-c', 'core.autocrlf=false', '-c', 'core.eol=lf', '-C', $source, 'apply', '--whitespace=error-all', $patch)
    foreach ($entry in $inputs.patchedFiles.PSObject.Properties) { Assert-Hash (Join-Path $source $entry.Name) $entry.Value }
    foreach ($name in $requiredLocks) {
        $project = $name -replace '\.lock\.json$', ''
        Copy-Item -LiteralPath (Join-Path $payload ('locks/' + $name)) -Destination (Join-Path $source ($project + '/packages.lock.json'))
    }
    $projectPath = Join-Path $source 'Il2CppInterop.Generator/Il2CppInterop.Generator.csproj'
    Run $DotNetPath @('restore', $projectPath, '--locked-mode', '--configfile', (Join-Path $payload 'NuGet.Config'), '-p:RestorePackagesWithLockFile=true')
    Run $DotNetPath @('build', $projectPath, '-c', 'Release', '-f', $inputs.targetFramework, '--no-restore', '-p:GeneratePackageOnBuild=false', '-p:ContinuousIntegrationBuild=true')
    foreach ($name in $requiredLocks) {
        $project = $name -replace '\.lock\.json$', ''
        Assert-Hash (Join-Path $source ($project + '/packages.lock.json')) $inputs.lockFiles.PSObject.Properties[$name].Value
    }
    foreach ($entry in $inputs.preservedFiles.PSObject.Properties) { Assert-Hash (Join-Path $source $entry.Name) $entry.Value }
    foreach ($entry in $inputs.patchedFiles.PSObject.Properties) { Assert-Hash (Join-Path $source $entry.Name) $entry.Value }
    $outputDirectory = Join-Path $source ('bin/Il2CppInterop.Generator/' + $inputs.targetFramework)
    $generator = Join-Path $outputDirectory 'Il2CppInterop.Generator.dll'
    $identity = Assert-Assembly $generator 'Il2CppInterop.Generator'
    $commonIdentity = Assert-Assembly (Join-Path $outputDirectory 'Il2CppInterop.Common.dll') 'Il2CppInterop.Common'
    $stage = Join-Path $destinationPath 'stage'
    New-Item -ItemType Directory -Path $stage | Out-Null
    Copy-Item -LiteralPath $generator -Destination $stage
    foreach ($entry in $inputs.licenceFiles.PSObject.Properties) { Copy-Item -LiteralPath (Join-Path $payload $entry.Name) -Destination $stage }
    $correspondingSource = Join-Path $stage 'corresponding-source'
    New-Item -ItemType Directory -Path $correspondingSource | Out-Null
    Copy-Item -LiteralPath $archive -Destination (Join-Path $correspondingSource 'source.zip')
    foreach ($name in @('pointer-byref.patch', 'Build-Generator.ps1', 'build-inputs.json', 'NuGet.Config', 'UPSTREAM-LICENSE', 'MODIFICATIONS-LICENSE')) {
        Copy-Item -LiteralPath (Join-Path $payload $name) -Destination $correspondingSource
    }
    Copy-Item -LiteralPath (Join-Path $payload 'locks') -Destination $correspondingSource -Recurse
    [ordered]@{
        sourceCommit = $inputs.sourceCommit
        sourceSha256 = $inputs.sourceSha256
        patchSha256 = $inputs.patchSha256
        nugetConfigSha256 = $inputs.nugetConfigSha256
        lockFiles = $inputs.lockFiles
        sdk = $sdkVersion
        targetFramework = $inputs.targetFramework
        outputs = @([ordered]@{ file = 'Il2CppInterop.Generator.dll'; sha256 = (Get-FileHash -LiteralPath $generator).Hash; identity = $identity })
        builtCommonIdentity = $commonIdentity
        installed = $false
    } | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath (Join-Path $stage 'build-manifest.json')
    Write-Output "Generator staged at $stage. Nothing installed in Rust; native generation and loader verification remain required."
} finally {
    if ($sourceLocationPushed) { Pop-Location }
    foreach ($name in $environmentNames) {
        if ($null -eq $priorEnvironment[$name]) { Remove-Item -LiteralPath ('Env:' + $name) -ErrorAction SilentlyContinue }
        else { [Environment]::SetEnvironmentVariable($name, $priorEnvironment[$name], 'Process') }
    }
    Stop-Transcript | Out-Null
}

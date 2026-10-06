# Project-authored build recipe. Pinned upstream changes retain the upstream MIT licence.
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
$patch = Join-Path $payload 'property-signatures.patch'
Assert-Hash $patch $inputs.patchSha256
foreach ($entry in $inputs.lockFiles.PSObject.Properties) {
    Assert-Hash (Join-Path $payload ('locks/' + $entry.Name)) $entry.Value
}
Assert-Hash (Join-Path $payload 'NuGet.Config') $inputs.nugetConfigSha256
if (![IO.Path]::IsPathFullyQualified($Destination)) { throw 'Destination must be an absolute filesystem path.' }
$destinationPath = [IO.Path]::GetFullPath($Destination)
if (Test-Path -LiteralPath $destinationPath) { throw 'Destination must be a new directory; existing files are preserved.' }
if ($SourceArchive) {
    $SourceArchive = (Resolve-Path -LiteralPath $SourceArchive).Path
    Assert-Hash $SourceArchive $inputs.sourceSha256
}
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
    # Portable tools stay inside this build directory. No installer, profile or game change.
    if (!$DotNetPath) {
        if (!$IsWindows) { throw 'Supply an explicit path to SDK 9.0.318 dotnet on this platform.' }
        $sdkArchive = Join-Path $destinationPath 'sdk.zip'
        Invoke-WebRequest -Uri $inputs.windowsSdk.url -OutFile $sdkArchive
        Assert-Hash $sdkArchive $inputs.windowsSdk.hash 'SHA512'
        $sdkDirectory = Join-Path $destinationPath 'sdk'
        Expand-Archive -LiteralPath $sdkArchive -DestinationPath $sdkDirectory
        $DotNetPath = Join-Path $sdkDirectory 'dotnet.exe'
    }
    $DotNetPath = (Resolve-Path -LiteralPath $DotNetPath).Path
    if (!$GitPath) {
        $gitCommand = Get-Command git -CommandType Application -ErrorAction SilentlyContinue
        if ($gitCommand) { $GitPath = $gitCommand.Source }
        elseif ($IsWindows) {
            $gitArchive = Join-Path $destinationPath 'mingit.zip'
            Invoke-WebRequest -Uri $inputs.windowsGit.url -OutFile $gitArchive
            Assert-Hash $gitArchive $inputs.windowsGit.sha256
            $gitDirectory = Join-Path $destinationPath 'git'
            Expand-Archive -LiteralPath $gitArchive -DestinationPath $gitDirectory
            $GitPath = Join-Path $gitDirectory 'cmd/git.exe'
        } else { throw 'Supply a path to Git.' }
    }
    $GitPath = (Resolve-Path -LiteralPath $GitPath).Path
    Run $GitPath @('--version')
    $archive = Join-Path $destinationPath 'source.zip'
    if ($SourceArchive) { Copy-Item -LiteralPath $SourceArchive -Destination $archive }
    else { Invoke-WebRequest -Uri $inputs.sourceUrl -OutFile $archive }
    Assert-Hash $archive $inputs.sourceSha256
    $unpacked = Join-Path $destinationPath 'upstream'
    Expand-Archive -LiteralPath $archive -DestinationPath $unpacked
    $source = Join-Path $unpacked ('Cpp2IL-' + $inputs.sourceCommit)
    if (!(Test-Path -LiteralPath (Join-Path $source 'Cpp2IL.Core/Cpp2IL.Core.csproj'))) { throw 'Unexpected source archive layout.' }
    Push-Location -LiteralPath $source
    $sourceLocationPushed = $true
    $sdkVersion = (& $DotNetPath --version | Out-String).Trim()
    if ($LASTEXITCODE -ne 0 -or $sdkVersion -cne $inputs.sdkVersion) { throw "Expected SDK $($inputs.sdkVersion); got $sdkVersion" }
    if (Test-Path -LiteralPath (Join-Path $source '.git')) { throw 'Source archive unexpectedly contains Git state.' }
    Run $GitPath @('-C', $source, 'init', '--quiet')
    Run $GitPath @('-C', $source, 'apply', '--check', '--whitespace=error-all', $patch)
    Run $GitPath @('-C', $source, 'apply', '--whitespace=error-all', $patch)
    foreach ($entry in $inputs.patchedFiles.PSObject.Properties) { Assert-Hash (Join-Path $source $entry.Name) $entry.Value }
    foreach ($entry in $inputs.lockFiles.PSObject.Properties) {
        $project = $entry.Name -replace '\.lock\.json$', ''
        Copy-Item -LiteralPath (Join-Path $payload ('locks/' + $entry.Name)) -Destination (Join-Path $source ($project + '/packages.lock.json'))
    }
    $projectPath = Join-Path $source 'Cpp2IL.Core/Cpp2IL.Core.csproj'
    # Restore the original framework graph; each project has its own lock file.
    Run $DotNetPath @('restore', $projectPath, '--locked-mode', '--configfile', (Join-Path $payload 'NuGet.Config'), '-p:RestorePackagesWithLockFile=true', '-p:ImportDirectoryBuildProps=false', '-p:ImportDirectoryBuildTargets=false')
    Run $DotNetPath @('build', $projectPath, '-c', 'Release', '-f', $inputs.targetFramework, '--no-restore', '-p:GeneratePackageOnBuild=false', '-p:ImportDirectoryBuildProps=false', '-p:ImportDirectoryBuildTargets=false', '-p:ContinuousIntegrationBuild=true', '-p:IncludeSourceRevisionInInformationalVersion=false', ('-p:InformationalVersion=' + $inputs.informationalVersion))
    foreach ($entry in $inputs.lockFiles.PSObject.Properties) {
        $project = $entry.Name -replace '\.lock\.json$', ''
        Assert-Hash (Join-Path $source ($project + '/packages.lock.json')) $entry.Value
    }
    $stage = Join-Path $destinationPath 'stage'
    New-Item -ItemType Directory -Path $stage | Out-Null
    $outputs = @()
    foreach ($name in @('Cpp2IL.Core', 'LibCpp2IL')) {
        $dll = Join-Path $source ('Cpp2IL.Core/bin/Release/' + $inputs.targetFramework + '/' + $name + '.dll')
        $identity = [Reflection.AssemblyName]::GetAssemblyName($dll)
        if ($identity.Name -cne $name -or $identity.Version.ToString() -cne $inputs.assemblyVersion -or $identity.GetPublicKeyToken().Length -ne 0 -or $identity.CultureName) { throw "Unexpected assembly identity: $name" }
        Copy-Item -LiteralPath $dll -Destination $stage
        $outputs += [ordered]@{ file = $name + '.dll'; sha256 = (Get-FileHash -LiteralPath $dll).Hash; identity = $identity.FullName }
    }
    # Check identities of built dependencies without staging replacements for them.
    foreach ($dependency in @(@('StableNameDotNet', '0.1.0.0'), @('WasmDisassembler', '2022.1.0.0'))) {
        $dll = Join-Path $source ('Cpp2IL.Core/bin/Release/' + $inputs.targetFramework + '/' + $dependency[0] + '.dll')
        if ([Reflection.AssemblyName]::GetAssemblyName($dll).Version.ToString() -cne $dependency[1]) { throw "Dependency identity changed: $($dependency[0])" }
    }
    foreach ($licence in @('UPSTREAM-LICENSE', 'MODIFICATIONS-LICENSE')) { Copy-Item -LiteralPath (Join-Path $payload $licence) -Destination $stage }
    [ordered]@{ sourceCommit = $inputs.sourceCommit; sourceSha256 = $inputs.sourceSha256; patchSha256 = $inputs.patchSha256; sdk = $sdkVersion; targetFramework = $inputs.targetFramework; informationalVersion = $inputs.informationalVersion; outputs = $outputs; installed = $false } | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath (Join-Path $stage 'build-manifest.json')
    Write-Output "Build staged at $stage. Nothing installed in Rust; native generation and loader verification remain required."
} finally {
    if ($sourceLocationPushed) { Pop-Location }
    foreach ($name in $environmentNames) { [Environment]::SetEnvironmentVariable($name, $priorEnvironment[$name], 'Process') }
    Stop-Transcript | Out-Null
}

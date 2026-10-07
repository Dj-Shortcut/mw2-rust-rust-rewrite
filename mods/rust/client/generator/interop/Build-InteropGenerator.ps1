#Requires -Version 7.4
[CmdletBinding()]
param(
    [Parameter(Mandatory)][string]$Destination,
    [Parameter(Mandatory)][string]$OfficialDependencyPath,
    [string]$DotNetPath,
    [string]$GitPath,
    [string]$SourceArchive,
    [string]$InputsPath = (Join-Path $PSScriptRoot 'build-inputs.json')
)
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Local-Absolute([string]$Path) {
    if (![IO.Path]::IsPathFullyQualified($Path) -or $Path.StartsWith('\\') -or $Path.StartsWith('//')) {
        throw 'An absolute local-volume path is required.'
    }
    $full = [IO.Path]::GetFullPath($Path)
    if ($IsWindows -and ($full.Length -lt 3 -or $full[0] -cnotmatch '[A-Za-z]' -or $full[1] -ne ':' -or $full[2] -ne [IO.Path]::DirectorySeparatorChar)) {
        throw 'UNC and device paths are not allowed.'
    }
    return $full
}
function Assert-NoReparseAncestors([string]$Path) {
    $cursor = $Path
    while ($cursor) {
        if (Test-Path -LiteralPath $cursor) {
            $item = Get-Item -LiteralPath $cursor -Force
            if (($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) { throw "Reparse-point path component: $cursor" }
        }
        $parent = [IO.Path]::GetDirectoryName($cursor)
        if (!$parent -or $parent -ceq $cursor) { break }
        $cursor = $parent
    }
}
function Existing-File([string]$Path) {
    $full = Local-Absolute $Path
    Assert-NoReparseAncestors $full
    if (!(Test-Path -LiteralPath $full -PathType Leaf)) { throw "Required file missing: $full" }
    return $full
}
function Existing-Directory([string]$Path) {
    $full = Local-Absolute $Path
    Assert-NoReparseAncestors $full
    if (!(Test-Path -LiteralPath $full -PathType Container)) { throw "Required directory missing: $full" }
    return $full
}
function Within([string]$Child, [string]$Parent) {
    $comparison = if ($IsWindows) { [StringComparison]::OrdinalIgnoreCase } else { [StringComparison]::Ordinal }
    $prefix = $Parent.TrimEnd([IO.Path]::DirectorySeparatorChar, [IO.Path]::AltDirectorySeparatorChar) + [IO.Path]::DirectorySeparatorChar
    return $Child.Equals($Parent, $comparison) -or $Child.StartsWith($prefix, $comparison)
}
function Assert-Hash([string]$Path, [string]$Expected, [string]$Algorithm = 'SHA256') {
    $length = if ($Algorithm -ceq 'SHA512') { 128 } else { 64 }
    if ($Expected -cnotmatch ('^[0-9a-fA-F]{' + $length + '}$')) { throw "Missing or malformed digest for $Path" }
    if ((Get-FileHash -LiteralPath $Path -Algorithm $Algorithm).Hash -ine $Expected) { throw "Hash mismatch: $Path" }
}
function Run([string]$Executable, [string[]]$Arguments) {
    & $Executable @Arguments
    if ($LASTEXITCODE -ne 0) { throw "Command failed ($LASTEXITCODE): $Executable" }
}
function New-Text([string]$Path, [string]$Text) {
    $stream = [IO.File]::Open($Path, [IO.FileMode]::CreateNew, [IO.FileAccess]::Write)
    try { $writer = [IO.StreamWriter]::new($stream, [Text.UTF8Encoding]::new($false)); try { $writer.Write($Text) } finally { $writer.Dispose() } }
    finally { $stream.Dispose() }
}
function Source-Snapshot([string]$Root) {
    $snapshot = [Collections.Generic.Dictionary[string,string]]::new([StringComparer]::Ordinal)
    foreach ($item in Get-ChildItem -LiteralPath $Root -Recurse -Force) {
        $relative = [IO.Path]::GetRelativePath($Root, $item.FullName).Replace('\', '/')
        if ($relative -match '(^|/)(obj|bin|\.git)(/|$)') { continue }
        if (($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) { throw 'Source tree contains a reparse point.' }
        if (!$item.PSIsContainer) { $snapshot.Add($relative, (Get-FileHash -LiteralPath $item.FullName).Hash) }
    }
    return ,$snapshot
}
function Assert-SameSnapshot($Before, $After) {
    if ($Before.Count -ne $After.Count) { throw 'Prepared source inventory changed during restore/build.' }
    foreach ($key in $Before.Keys) {
        if (!$After.ContainsKey($key) -or $Before[$key] -cne $After[$key]) { throw "Prepared source changed: $key" }
    }
}
function Expand-Source([string]$Archive, [string]$Output, [string]$ExpectedRoot) {
    $zip = [IO.Compression.ZipFile]::OpenRead($Archive)
    try {
        if ($zip.Entries.Count -gt 10000) { throw 'Source archive entry limit exceeded.' }
        $seen = [Collections.Generic.HashSet[string]]::new([StringComparer]::OrdinalIgnoreCase)
        [long]$expandedBytes = 0
        foreach ($entry in $zip.Entries) {
            $name = $entry.FullName
            $parts = $name.Split('/')
            if ($name.Contains('\') -or $name.StartsWith('/') -or $name.Contains(':') -or $parts[0] -cne $ExpectedRoot -or $parts -contains '..' -or $parts -contains '.') { throw 'Unsafe source archive path.' }
            if (!$seen.Add($name)) { throw 'Duplicate source archive path.' }
            if ((($entry.ExternalAttributes -shr 16) -band 0xF000) -eq 0xA000 -or ($entry.ExternalAttributes -band 0x400) -ne 0) { throw 'Source archive contains a link.' }
            $expandedBytes += $entry.Length
            if ($expandedBytes -gt 268435456) { throw 'Source archive expanded-size limit exceeded.' }
        }
    } finally { $zip.Dispose() }
    Expand-Archive -LiteralPath $Archive -DestinationPath $Output
}

if ('InteropGeneratorRecipe.Metadata' -as [type]) { throw 'Run this recipe once in a fresh PowerShell process; an existing metadata helper must not be reused.' }
# Use PowerShell's shipped default reference assemblies, not runtime TPA DLLs.
# Explicit ReferencedAssemblies replaces that compiler reference set.
if (!(Test-Path -LiteralPath (Join-Path $PSHOME 'ref') -PathType Container)) { throw 'The shipped PowerShell compiler reference directory is required.' }
Add-Type -TypeDefinition @'
using System;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Reflection.Metadata;
using System.Reflection.PortableExecutable;
namespace InteropGeneratorRecipe {
    public sealed class Info {
        public string Identity { get; set; }
        public string TargetFramework { get; set; }
        public string[] References { get; set; }
        public uint DefinitionFlags { get; set; }
        public string[] ReferenceBindings { get; set; }
        public byte[] PublicKeyToken { get; set; }
    }
    public static class Metadata {
        static AssemblyName Name(MetadataReader r, StringHandle n, Version v, StringHandle c, BlobHandle k, AssemblyFlags flags, bool fullKey) {
            var a = new AssemblyName { Name = r.GetString(n), Version = v, CultureName = c.IsNil ? "" : r.GetString(c),
                Flags = (AssemblyNameFlags)(uint)flags, ContentType = (AssemblyContentType)(((uint)flags & 0x0e00u) >> 9) };
            var bytes = k.IsNil ? Array.Empty<byte>() : r.GetBlobBytes(k);
            if (fullKey && bytes.Length != 0) a.SetPublicKey(bytes); else a.SetPublicKeyToken(bytes);
            return a;
        }
        static bool FrameworkAttribute(MetadataReader r, EntityHandle constructor) {
            if (constructor.Kind != HandleKind.MemberReference) return false;
            var member = r.GetMemberReference((MemberReferenceHandle)constructor);
            if (r.GetString(member.Name) != ".ctor" || member.Parent.Kind != HandleKind.TypeReference) return false;
            var type = r.GetTypeReference((TypeReferenceHandle)member.Parent);
            return r.GetString(type.Namespace) == "System.Runtime.Versioning" && r.GetString(type.Name) == "TargetFrameworkAttribute";
        }
        public static Info Read(string path) {
            using (var stream = File.OpenRead(path)) using (var pe = new PEReader(stream)) {
                if (!pe.HasMetadata) throw new BadImageFormatException("Managed PE metadata required.");
                var reader = pe.GetMetadataReader();
                if (!reader.IsAssembly) throw new BadImageFormatException("Assembly metadata required.");
                var a = reader.GetAssemblyDefinition();
                var name = Name(reader, a.Name, a.Version, a.Culture, a.PublicKey, a.Flags, true);
                string framework = null;
                foreach (var handle in a.GetCustomAttributes()) {
                    var attr = reader.GetCustomAttribute(handle);
                    if (!FrameworkAttribute(reader, attr.Constructor)) continue;
                    if (framework != null) throw new BadImageFormatException("Duplicate target framework attribute.");
                    var blob = reader.GetBlobReader(attr.Value);
                    if (blob.ReadUInt16() != 1) throw new BadImageFormatException("Malformed target framework attribute.");
                    framework = blob.ReadSerializedString();
                }
                if (String.IsNullOrEmpty(framework)) throw new BadImageFormatException("Target framework attribute required.");
                var refs = reader.AssemblyReferences.Select(handle => {
                    var r = reader.GetAssemblyReference(handle);
                    return new { Identity = Name(reader, r.Name, r.Version, r.Culture, r.PublicKeyOrToken, r.Flags, (r.Flags & AssemblyFlags.PublicKey) != 0).FullName, Flags = (uint)r.Flags };
                }).OrderBy(x => x.Identity, StringComparer.Ordinal).ToArray();
                if (refs.Select(x => x.Identity).Distinct(StringComparer.Ordinal).Count() != refs.Length) throw new BadImageFormatException("Duplicate assembly reference identity.");
                return new Info { Identity = name.FullName, DefinitionFlags = (uint)a.Flags, TargetFramework = framework,
                    References = refs.Select(x => x.Identity).ToArray(), ReferenceBindings = refs.Select(x => x.Identity + " | RawFlags=" + x.Flags.ToString("X8")).ToArray(), PublicKeyToken = name.GetPublicKeyToken() };
            }
        }
    }
}
'@

$inputsFile = Existing-File $InputsPath
$payload = Existing-Directory ([IO.Path]::GetDirectoryName($inputsFile))
$inputs = Get-Content -LiteralPath $inputsFile -Raw | ConvertFrom-Json
if ($inputs.schemaVersion -ne 1 -or $inputs.sourceCommit -cne 'dbda1cb353b0f4253345dc45136d170b9e50a5a0' -or
    $inputs.sourceUrl -cne 'https://codeload.github.com/BepInEx/Il2CppInterop/zip/dbda1cb353b0f4253345dc45136d170b9e50a5a0' -or
    $inputs.sourceSha256 -cne '96d223a475c885ef08118a3adaee0c2ed5d65edd83fbea9987e5a305f1123ac5' -or
    $inputs.sdkVersion -cne '9.0.318' -or $inputs.targetFramework -cne 'netstandard2.1' -or
    $inputs.commonTargetFramework -cne 'netstandard2.0' -or $inputs.assemblyVersion -cne '1.5.3.0' -or
    $inputs.modificationLicense -cne 'LGPL-3.0-only') { throw 'Unexpected recipe profile.' }
$expectedPatchFiles = @('Il2CppInterop.Generator/Extensions/ILGeneratorEx.cs', 'Il2CppInterop.Generator/Passes/Pass50GenerateMethods.cs')
$expectedLocks = @('Il2CppInterop.Generator.lock.json', 'Il2CppInterop.Common.lock.json')
$expectedOfficial = @('Il2CppInterop.Generator.dll', 'Il2CppInterop.Common.dll', 'AsmResolver.dll', 'AsmResolver.PE.dll', 'AsmResolver.PE.File.dll', 'AsmResolver.DotNet.dll', '0Harmony.dll')
foreach ($pair in @(@($inputs.patchedFiles, $expectedPatchFiles), @($inputs.lockFiles, $expectedLocks), @($inputs.officialDependencies, $expectedOfficial))) {
    $actual = @($pair[0].PSObject.Properties.Name)
    if (@(Compare-Object -ReferenceObject $pair[1] -DifferenceObject $actual -CaseSensitive).Count -ne 0) { throw 'Unexpected manifest file set.' }
}
$officialRoot = Existing-Directory $OfficialDependencyPath
$destinationPath = Local-Absolute $Destination
Assert-NoReparseAncestors $destinationPath
if (Test-Path -LiteralPath $destinationPath) { throw 'Destination must be new; existing files are preserved.' }
$destinationParent = Existing-Directory ([IO.Path]::GetDirectoryName($destinationPath))
foreach ($readOnlyRoot in @($payload, $officialRoot)) {
    if ((Within $destinationPath $readOnlyRoot) -or (Within $readOnlyRoot $destinationPath)) { throw 'Destination overlaps a read-only input tree.' }
}
for ($ancestor = $destinationParent; $ancestor; $ancestor = [IO.Path]::GetDirectoryName($ancestor)) {
    foreach ($marker in @('RustClient.exe', 'RustDedicated.exe', 'GameAssembly.dll', 'UnityPlayer.dll', 'RustClient_Data', 'RustDedicated_Data')) {
        if (Test-Path -LiteralPath (Join-Path $ancestor $marker)) { throw 'Destination must be outside any game installation.' }
    }
}
$pins = [Collections.Generic.Dictionary[string,string]]::new([StringComparer]::Ordinal)
$pins.Add($inputsFile, (Get-FileHash -LiteralPath $inputsFile).Hash)
$pins.Add((Existing-File $PSCommandPath), (Get-FileHash -LiteralPath $PSCommandPath).Hash)
$patch = Existing-File (Join-Path $payload 'pointer-byref.patch')
Assert-Hash $patch $inputs.patchSha256
$pins.Add($patch, $inputs.patchSha256)
foreach ($entry in $inputs.lockFiles.PSObject.Properties) {
    $file = Existing-File (Join-Path $payload ('locks/' + $entry.Name)); Assert-Hash $file $entry.Value; $pins.Add($file, $entry.Value)
}
$nuget = Existing-File (Join-Path $payload 'NuGet.Config')
Assert-Hash $nuget $inputs.nugetConfigSha256; $pins.Add($nuget, $inputs.nugetConfigSha256)
foreach ($licence in @(@('UPSTREAM-LICENSE', $inputs.upstreamLicenseSha256), @('MODIFICATIONS-LICENSE', $inputs.modificationsLicenseSha256), @('MODIFICATION-NOTICE.md', $inputs.modificationNoticeSha256))) {
    $file = Existing-File (Join-Path $payload $licence[0]); Assert-Hash $file $licence[1]; $pins.Add($file, $licence[1])
}
$officialInfo = @{}
foreach ($entry in $inputs.officialDependencies.PSObject.Properties) {
    $file = Existing-File (Join-Path $officialRoot $entry.Name)
    Assert-Hash $file $entry.Value.sha256; $pins.Add($file, $entry.Value.sha256)
    $info = [InteropGeneratorRecipe.Metadata]::Read($file)
    if ($info.Identity -cne $entry.Value.identity) { throw "Official dependency identity mismatch: $($entry.Name)" }
    $expectedToken = $entry.Value.publicKeyTokenHex
    if ($expectedToken -isnot [string] -or $expectedToken -cnotmatch '^([0-9a-f]{16})?$') { throw 'Manifest public-key token must be an exact lowercase token or the measured empty string.' }
    $observedToken = [Convert]::ToHexString($info.PublicKeyToken).ToLowerInvariant()
    if ($observedToken -cne $expectedToken) { throw "Official dependency public-key token mismatch: $($entry.Name)" }
    if ($entry.Value.targetFramework -isnot [string] -or $info.TargetFramework -cne $entry.Value.targetFramework) { throw "Official dependency target framework mismatch: $($entry.Name)" }
    $expectedFlags = $entry.Value.definitionFlags
    if (($expectedFlags -isnot [int] -and $expectedFlags -isnot [long]) -or $expectedFlags -lt 0 -or $expectedFlags -gt [uint32]::MaxValue -or $info.DefinitionFlags -ne $expectedFlags) { throw "Official dependency definition flags mismatch: $($entry.Name)" }
    if ($entry.Name -in @('Il2CppInterop.Generator.dll', 'Il2CppInterop.Common.dll')) {
        $expectedIdentity = [IO.Path]::GetFileNameWithoutExtension($entry.Name) + ', Version=1.5.3.0, Culture=neutral, PublicKeyToken=null'
        if ($info.Identity -cne $expectedIdentity -or $observedToken -cne '') { throw 'The official Generator/Common identity is outside the pinned upstream profile.' }
    }
    $officialInfo[$entry.Name] = $info
}
foreach ($provided in @($DotNetPath, $GitPath, $SourceArchive)) {
    if ($provided) { $file = Existing-File $provided; if (!$pins.ContainsKey($file)) { $pins.Add($file, (Get-FileHash -LiteralPath $file).Hash) } }
}
if ($SourceArchive) { $SourceArchive = Existing-File $SourceArchive; Assert-Hash $SourceArchive $inputs.sourceSha256 }

$gitEnvironmentNames = @('GIT_DIR', 'GIT_WORK_TREE', 'GIT_INDEX_FILE', 'GIT_COMMON_DIR', 'GIT_OBJECT_DIRECTORY', 'GIT_ALTERNATE_OBJECT_DIRECTORIES', 'GIT_CONFIG', 'GIT_CONFIG_PARAMETERS', 'GIT_CONFIG_COUNT', 'GIT_CONFIG_SYSTEM', 'GIT_CONFIG_GLOBAL', 'GIT_CONFIG_NOSYSTEM', 'GIT_NAMESPACE', 'GIT_TEMPLATE_DIR', 'GIT_EXEC_PATH')
$environmentNames = @('DOTNET_CLI_HOME', 'NUGET_PACKAGES', 'DOTNET_SKIP_FIRST_TIME_EXPERIENCE', 'DOTNET_CLI_TELEMETRY_OPTOUT', 'DOTNET_GENERATE_ASPNET_CERTIFICATE') + $gitEnvironmentNames
$priorEnvironment = @{}
foreach ($name in $environmentNames) { $priorEnvironment[$name] = [Environment]::GetEnvironmentVariable($name, 'Process') }
$sourceLocationPushed = $false
$transcriptStarted = $false
$preparedSnapshot = $null
$source = $null
$failure = $null
$cleanupFailures = [Collections.Generic.List[string]]::new()
$successfulInputRechecks = 0
$preparedSourceRechecked = $false
$stageGenerator = $null
$stageGeneratorHash = $null
$stagedGeneratorRechecked = $false
$built = $null
$sdkProfile = '{"sdk":{"version":"9.0.318","rollForward":"disable"}}'
New-Item -ItemType Directory -Path $destinationPath | Out-Null
try {
    Start-Transcript -LiteralPath (Join-Path $destinationPath 'build-transcript.txt') -NoClobber | Out-Null
    $transcriptStarted = $true
    $env:DOTNET_CLI_HOME = Join-Path $destinationPath 'dotnet-home'
    $env:NUGET_PACKAGES = Join-Path $destinationPath 'packages'
    $env:DOTNET_SKIP_FIRST_TIME_EXPERIENCE = '1'
    $env:DOTNET_CLI_TELEMETRY_OPTOUT = '1'
    $env:DOTNET_GENERATE_ASPNET_CERTIFICATE = 'false'
    # -C alone cannot fence Git if its inherited repository/config variables
    # redirect it. Isolate these process-only values and restore them in finally.
    foreach ($name in $gitEnvironmentNames) { [Environment]::SetEnvironmentVariable($name, $null, 'Process') }
    $emptyGitConfig = Join-Path $destinationPath 'empty-git-config'
    New-Text $emptyGitConfig ''
    $env:GIT_CONFIG_NOSYSTEM = '1'
    $env:GIT_CONFIG_GLOBAL = $emptyGitConfig
    if (!$DotNetPath) {
        if (!$IsWindows) { throw 'Supply an explicit SDK 9.0.318 dotnet path on this platform.' }
        if ($inputs.windowsSdk.url -cne 'https://builds.dotnet.microsoft.com/dotnet/Sdk/9.0.318/dotnet-sdk-9.0.318-win-x64.zip' -or
            $inputs.windowsSdk.hash -ine 'd961a84ebcbc316f06c98d16958422f7bb1b3a1f29b5d6011cabf59984390690ff669c3dd9b739554ad00771d6393dbbde6bdb14da81faa7fe499f59501864b6') { throw 'Unexpected portable SDK pin.' }
        $sdkZip = Join-Path $destinationPath 'sdk.zip'; Invoke-WebRequest -Uri $inputs.windowsSdk.url -OutFile $sdkZip
        Assert-Hash $sdkZip $inputs.windowsSdk.hash 'SHA512'
        $pins.Add($sdkZip, (Get-FileHash -LiteralPath $sdkZip).Hash)
        Expand-Archive -LiteralPath $sdkZip -DestinationPath (Join-Path $destinationPath 'sdk')
        $DotNetPath = Join-Path $destinationPath 'sdk/dotnet.exe'
    }
    $DotNetPath = Existing-File $DotNetPath
    if (!$GitPath) {
        if (!$IsWindows) { throw 'Supply an explicit Git path on this platform.' }
        if ($inputs.windowsGit.url -cne 'https://github.com/git-for-windows/git/releases/download/v2.56.0.windows.2/MinGit-2.56.0.2-64-bit.zip' -or
            $inputs.windowsGit.sha256 -ine 'da35e72aa21c005a5a0d298cfbae110bc1609a815730ea0dde84b01a1b3cd3be') { throw 'Unexpected portable Git pin.' }
        $gitZip = Join-Path $destinationPath 'mingit.zip'; Invoke-WebRequest -Uri $inputs.windowsGit.url -OutFile $gitZip
        Assert-Hash $gitZip $inputs.windowsGit.sha256
        $pins.Add($gitZip, $inputs.windowsGit.sha256)
        Expand-Archive -LiteralPath $gitZip -DestinationPath (Join-Path $destinationPath 'git')
        $GitPath = Join-Path $destinationPath 'git/cmd/git.exe'
    }
    $GitPath = Existing-File $GitPath
    $gitVersion = (& $GitPath --version | Out-String).Trim()
    if ($LASTEXITCODE -ne 0) { throw 'Git executable failed.' }
    $archive = Join-Path $destinationPath 'source.zip'
    if ($SourceArchive) { Copy-Item -LiteralPath $SourceArchive -Destination $archive }
    else { Invoke-WebRequest -Uri $inputs.sourceUrl -OutFile $archive }
    Assert-Hash $archive $inputs.sourceSha256
    $pins.Add($archive, $inputs.sourceSha256)
    $sourceRootName = 'Il2CppInterop-' + $inputs.sourceCommit
    $unpacked = Join-Path $destinationPath 'upstream'
    Expand-Source $archive $unpacked $sourceRootName
    $source = Existing-Directory (Join-Path $unpacked $sourceRootName)
    $generatorProject = Existing-File (Join-Path $source 'Il2CppInterop.Generator/Il2CppInterop.Generator.csproj')
    $null = Existing-File (Join-Path $source 'Il2CppInterop.Common/Il2CppInterop.Common.csproj')
    Assert-Hash (Join-Path $source 'LICENSE') $inputs.upstreamLicenseSha256
    $originalSnapshot = Source-Snapshot $source
    $originalSdkProfileHash = (Get-FileHash -LiteralPath (Join-Path $source 'global.json')).Hash
    if (Test-Path -LiteralPath (Join-Path $source '.git')) { throw 'Source archive unexpectedly contains Git state.' }
    Run $GitPath @('-C', $source, 'init', '--quiet')
    $patchStats = @(& $GitPath -c core.autocrlf=false -c core.eol=lf -C $source apply --numstat -- $patch)
    if ($LASTEXITCODE -ne 0) { throw 'Patch inspection failed.' }
    $patchNames = @($patchStats | ForEach-Object { ($_ -split "`t", 3)[2] })
    if (@(Compare-Object $expectedPatchFiles $patchNames -CaseSensitive).Count -ne 0) { throw 'Patch changes files outside the two claimed predicates.' }
    Run $GitPath @('-c', 'core.autocrlf=false', '-c', 'core.eol=lf', '-C', $source, 'apply', '--check', '--whitespace=error-all', '--', $patch)
    Run $GitPath @('-c', 'core.autocrlf=false', '-c', 'core.eol=lf', '-C', $source, 'apply', '--whitespace=error-all', '--', $patch)
    foreach ($entry in $inputs.patchedFiles.PSObject.Properties) { Assert-Hash (Join-Path $source $entry.Name) $entry.Value }
    [IO.File]::WriteAllText((Join-Path $source 'global.json'), $sdkProfile, [Text.UTF8Encoding]::new($false))
    foreach ($entry in $inputs.lockFiles.PSObject.Properties) {
        $project = $entry.Name -replace '\.lock\.json$', ''
        $lockDestination = Join-Path $source ($project + '/packages.lock.json')
        if (Test-Path -LiteralPath $lockDestination) { throw 'Upstream unexpectedly already contains a lock file.' }
        Copy-Item -LiteralPath (Join-Path $payload ('locks/' + $entry.Name)) -Destination $lockDestination
    }
    $preparedSnapshot = Source-Snapshot $source
    $allowedChanges = @($expectedPatchFiles) + @('global.json', 'Il2CppInterop.Generator/packages.lock.json', 'Il2CppInterop.Common/packages.lock.json')
    foreach ($key in @($originalSnapshot.Keys) + @($preparedSnapshot.Keys) | Sort-Object -Unique) {
        if ($key -notin $allowedChanges -and (!$originalSnapshot.ContainsKey($key) -or !$preparedSnapshot.ContainsKey($key) -or $originalSnapshot[$key] -cne $preparedSnapshot[$key])) { throw "Unexpected source preparation change: $key" }
    }
    Push-Location -LiteralPath $source
    $sourceLocationPushed = $true
    $sdkVersion = (& $DotNetPath --version | Out-String).Trim()
    if ($LASTEXITCODE -ne 0 -or $sdkVersion -cne $inputs.sdkVersion) { throw 'The pinned SDK profile was not selected.' }
    Run $DotNetPath @('restore', $generatorProject, '--locked-mode', '--configfile', $nuget, '-p:RestorePackagesWithLockFile=true')
    Run $DotNetPath @('build', $generatorProject, '-c', 'Release', '-f', 'netstandard2.1', '--no-restore', '-p:GeneratePackageOnBuild=false')
    foreach ($entry in $inputs.lockFiles.PSObject.Properties) {
        $project = $entry.Name -replace '\.lock\.json$', ''
        Assert-Hash (Join-Path $source ($project + '/packages.lock.json')) $entry.Value
    }
    $generatorDll = Existing-File (Join-Path $source 'bin/Il2CppInterop.Generator/netstandard2.1/Il2CppInterop.Generator.dll')
    $commonDll = Existing-File (Join-Path $source 'bin/Il2CppInterop.Common/netstandard2.0/Il2CppInterop.Common.dll')
    $comparisonRecords = @()
    foreach ($tuple in @(@('Il2CppInterop.Generator.dll', $generatorDll, '.NETStandard,Version=v2.1'), @('Il2CppInterop.Common.dll', $commonDll, '.NETStandard,Version=v2.0'))) {
        $observed = [InteropGeneratorRecipe.Metadata]::Read($tuple[1])
        $baseline = $officialInfo[$tuple[0]]
        if ($observed.Identity -cne $baseline.Identity -or $observed.DefinitionFlags -ne $baseline.DefinitionFlags -or $observed.TargetFramework -cne $tuple[2] -or $baseline.TargetFramework -cne $tuple[2] -or
            $observed.PublicKeyToken.Length -ne 0 -or @(Compare-Object $baseline.References $observed.References -CaseSensitive).Count -ne 0 -or
            @(Compare-Object $baseline.ReferenceBindings $observed.ReferenceBindings -CaseSensitive).Count -ne 0) { throw "Built metadata/dependency boundary differs from official baseline: $($tuple[0])" }
        $comparisonRecords += [ordered]@{ file = $tuple[0]; identity = $observed.Identity; definitionFlags = $observed.DefinitionFlags; targetFramework = $observed.TargetFramework; references = $observed.References; referenceBindings = $observed.ReferenceBindings; sha256 = (Get-FileHash -LiteralPath $tuple[1]).Hash; staged = ($tuple[0] -ceq 'Il2CppInterop.Generator.dll') }
    }
    $stage = Join-Path $destinationPath 'stage'
    New-Item -ItemType Directory -Path $stage | Out-Null
    $stageGenerator = Join-Path $stage 'Il2CppInterop.Generator.dll'
    $stageGeneratorHash = ($comparisonRecords | Where-Object { $_.file -ceq 'Il2CppInterop.Generator.dll' }).sha256
    Copy-Item -LiteralPath $generatorDll -Destination $stageGenerator
    Assert-Hash $stageGenerator $stageGeneratorHash
    foreach ($licence in @('UPSTREAM-LICENSE', 'MODIFICATIONS-LICENSE', 'MODIFICATION-NOTICE.md')) { Copy-Item -LiteralPath (Join-Path $payload $licence) -Destination (Join-Path $stage $licence) }
    $built = [ordered]@{ sourceCommit = $inputs.sourceCommit; sourceSha256 = $inputs.sourceSha256; patchSha256 = $inputs.patchSha256; sdkVersion = $sdkVersion; sdkProfile = $sdkProfile; originalSdkProfileSha256 = $originalSdkProfileHash; preparedSdkProfileSha256 = $preparedSnapshot['global.json']; gitVersion = $gitVersion; outputs = $comparisonRecords; installed = $false; scope = 'Generator source build and passive assembly identity/framework/reference comparison only.' }
} catch { $failure = $_ }
finally {
    foreach ($entry in $pins.GetEnumerator()) {
        try { Assert-NoReparseAncestors $entry.Key; Assert-Hash $entry.Key $entry.Value; $successfulInputRechecks++ }
        catch { $cleanupFailures.Add('input-recheck: ' + $_.Exception.Message) }
    }
    if ($preparedSnapshot -and $source) {
        try { Assert-SameSnapshot $preparedSnapshot (Source-Snapshot $source); $preparedSourceRechecked = $true }
        catch { $cleanupFailures.Add('prepared-source-recheck: ' + $_.Exception.Message) }
    }
    if ($stageGenerator -and $stageGeneratorHash) {
        try { Assert-NoReparseAncestors $stageGenerator; Assert-Hash $stageGenerator $stageGeneratorHash; $stagedGeneratorRechecked = $true }
        catch { $cleanupFailures.Add('staged-generator-recheck: ' + $_.Exception.Message) }
    }
    if ($sourceLocationPushed) {
        try { Pop-Location } catch { $cleanupFailures.Add('location-restore: ' + $_.Exception.Message) }
    }
    foreach ($name in $environmentNames) {
        try { [Environment]::SetEnvironmentVariable($name, $priorEnvironment[$name], 'Process') }
        catch { $cleanupFailures.Add('environment-restore: ' + $name) }
    }
    if ($transcriptStarted) { try { Stop-Transcript | Out-Null } catch { $cleanupFailures.Add('transcript-stop: ' + $_.Exception.Message) } }
}
$succeeded = $null -eq $failure -and $cleanupFailures.Count -eq 0 -and $null -ne $built -and $successfulInputRechecks -eq $pins.Count -and $preparedSourceRechecked -and $stagedGeneratorRechecked
$status = [ordered]@{ succeeded = $succeeded; inputRechecksAttempted = $pins.Count; inputRechecksSucceeded = $successfulInputRechecks; allInputHashesUnchanged = ($successfulInputRechecks -eq $pins.Count); preparedSourceRechecked = $preparedSourceRechecked; stagedGeneratorRechecked = $stagedGeneratorRechecked; cleanupFailures = @($cleanupFailures); installed = $false; failureType = $(if ($failure) { $failure.Exception.GetType().FullName } else { $null }); build = $built }
New-Text (Join-Path $destinationPath 'build-status.json') ($status | ConvertTo-Json -Depth 9)
if (!$succeeded) { throw 'Generator build/identity/preservation gate failed; fresh output retained and nothing installed.' }
New-Text (Join-Path $destinationPath 'stage/build-manifest.json') ($built | ConvertTo-Json -Depth 9)
Write-Output "Generator staged at $(Join-Path $destinationPath 'stage'). No generator execution, installation or gameplay verification occurred."

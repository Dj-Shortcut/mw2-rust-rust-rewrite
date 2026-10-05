[CmdletBinding()]
param(
    [string]$RootPath = '',
    [ValidatePattern('^(latest|[0-9]+\.[0-9]+\.[0-9]+)$')][string]$OxideRelease = 'latest'
)

function Get-NewServerRoot([string]$Path) {
    if (-not $Path) {
        if (-not $env:LOCALAPPDATA) { throw 'LOCALAPPDATA is unavailable; supply RootPath.' }
        $Path = Join-Path $env:LOCALAPPDATA 'CodexRustServer'
    }
    if ($Path -notmatch '^[A-Za-z]:\\' -or $Path -match '[\x00-\x1f"<>|?*]' -or $Path.Substring(2).Contains(':')) {
        throw 'RootPath must be an absolute local Windows drive path.'
    }
    $full = [IO.Path]::GetFullPath($Path).TrimEnd('\')
    $pathErrors = @()
    $existing = Get-Item -LiteralPath $full -Force -ErrorAction SilentlyContinue -ErrorVariable pathErrors
    if ($existing) { throw 'RootPath already exists; this installer never updates or overwrites an existing path.' }
    if (@($pathErrors | Where-Object { $_.CategoryInfo.Category -ne 'ObjectNotFound' }).Count) { throw 'RootPath absence could not be established safely.' }
    $parent = Split-Path -Parent $full
    if (-not (Test-Path -LiteralPath $parent -PathType Container)) { throw 'The parent directory must already exist.' }
    Assert-ServerAncestors $parent
    return $full
}

function Assert-ServerAncestors([string]$Path) {
    for ($probe = $Path; $probe;) {
        if (Test-Path -LiteralPath (Join-Path $probe '.git')) { throw 'RootPath cannot be inside a Git checkout.' }
        if ((Get-Item -LiteralPath $probe -Force).Attributes -band [IO.FileAttributes]::ReparsePoint) {
            throw 'RootPath cannot use a symbolic link or junction parent.'
        }
        $next = Split-Path -Parent $probe
        if (-not $next -or $next -eq $probe) { break }
        $probe = $next
    }
}

function Get-InstallFreeBytes([string]$Path) {
    $drive = [IO.DriveInfo]::new([IO.Path]::GetPathRoot($Path))
    if ($drive.DriveType -eq [IO.DriveType]::Network) { throw 'RootPath cannot use a mapped network drive.' }
    return $drive.AvailableFreeSpace
}

function Get-InstallFreeMemoryBytes {
    $memory = Get-CimInstance -ClassName Win32_OperatingSystem -ErrorAction Stop
    if ($null -eq $memory.FreePhysicalMemory -or [long]$memory.FreePhysicalMemory -le 0) { throw 'Windows did not report usable available memory.' }
    return [long]$memory.FreePhysicalMemory * 1KB
}

function Get-OxideMetadata([string]$Selector) {
    $suffix = if ($Selector -eq 'latest') { 'latest' } else { "tags/$Selector" }
    $headers = @{ Accept = 'application/vnd.github+json'; 'User-Agent' = 'CodexRustServer-Installer' }
    $release = Invoke-RestMethod -Uri "https://api.github.com/repos/OxideMod/Oxide.Rust/releases/$suffix" -Headers $headers -ErrorAction Stop
    if ($release.draft -or $release.prerelease -or $release.tag_name -notmatch '^[0-9]+\.[0-9]+\.[0-9]+$') {
        throw 'Only public stable Oxide.Rust releases are supported.'
    }
    if ($Selector -ne 'latest' -and $release.tag_name -ne $Selector) { throw 'The release tag does not match the selector.' }
    $assets = @($release.assets | Where-Object { $_.name -ceq 'Oxide.Rust.zip' -and $_.state -eq 'uploaded' })
    if ($assets.Count -ne 1) { throw 'The release must have exactly one uploaded Windows Oxide.Rust.zip asset.' }
    $url = "https://github.com/OxideMod/Oxide.Rust/releases/download/$($release.tag_name)/Oxide.Rust.zip"
    if ($assets[0].browser_download_url -cne $url) { throw 'The Oxide asset URL is not the official release URL.' }
    $digest = $assets[0].PSObject.Properties['digest']
    if (-not $digest -or $digest.Value -notmatch '^sha256:([a-fA-F0-9]{64})$') { throw 'The official Windows asset must publish a valid SHA-256 digest.' }
    $expected = $Matches[1].ToLowerInvariant()
    if ([long]$assets[0].size -le 0) { throw 'The official Windows asset must have a positive size.' }
    return @{ Release = $release; Asset = $assets[0]; Url = $url; ExpectedSha256 = $expected }
}

function Save-OfficialFile([string]$Url, [string]$Path) {
    Invoke-WebRequest -UseBasicParsing -Uri $Url -OutFile $Path -ErrorAction Stop
}

function Expand-OfficialArchive([string]$Zip, [string]$Destination) {
    Add-Type -AssemblyName System.IO.Compression.FileSystem
    $prefix = [IO.Path]::GetFullPath($Destination).TrimEnd('\') + '\'
    $archive = [IO.Compression.ZipFile]::OpenRead($Zip)
    try {
        foreach ($entry in $archive.Entries) {
            $target = [IO.Path]::GetFullPath([IO.Path]::Combine($Destination, $entry.FullName))
            if (-not $target.StartsWith($prefix, [StringComparison]::OrdinalIgnoreCase)) {
                throw 'The archive contains a path outside its destination.'
            }
        }
    } finally { $archive.Dispose() }
    Expand-Archive -LiteralPath $Zip -DestinationPath $Destination -Force -ErrorAction Stop
}

function Invoke-SteamInstall([string]$Executable, [string]$ServerRoot, [string]$LogRoot) {
    $arguments = @('+@ShutdownOnFailedCommand', '1', '+@NoPromptForPassword', '1', '+force_install_dir', ('"{0}"' -f $ServerRoot), '+login', 'anonymous', '+app_update', '258550', '-beta', 'public', 'validate', '+quit')
    $process = Start-Process -FilePath $Executable -ArgumentList $arguments -Wait -PassThru -NoNewWindow -WorkingDirectory (Split-Path -Parent $Executable) -RedirectStandardOutput (Join-Path $LogRoot 'steamcmd.stdout.log') -RedirectStandardError (Join-Path $LogRoot 'steamcmd.stderr.log') -ErrorAction Stop
    return $process.ExitCode
}

function Install-RustServer([string]$RootPath, [string]$OxideRelease = 'latest') {
    $ErrorActionPreference = 'Stop'
    Set-StrictMode -Version 2.0
    if ($env:OS -ne 'Windows_NT') { throw 'This installer supports Windows only.' }
    if ($OxideRelease -notmatch '^(latest|[0-9]+\.[0-9]+\.[0-9]+)$') { throw 'Use latest or a stable numeric Oxide release tag.' }
    $root = Get-NewServerRoot $RootPath
    if ((Get-InstallFreeBytes $root) -lt 20GB) { throw 'At least 20 GiB of free disk space is required.' }
    try { $freeMemory = Get-InstallFreeMemoryBytes } catch { throw 'Available RAM could not be determined through Windows CIM; installation refused.' }
    if ($freeMemory -lt 12GB) { throw 'At least 12 GiB of available RAM is required; close other applications or use a separate server machine.' }
    [Net.ServicePointManager]::SecurityProtocol = [Net.ServicePointManager]::SecurityProtocol -bor [Net.SecurityProtocolType]::Tls12
    $stage = 'release metadata'
    $created = $false
    $manifest = [ordered]@{ schemaVersion = 1; state = 'installing'; stage = 'preparation'; steamAppId = 258550; compatibility = 'unverified' }
    try {
        $metadata = Get-OxideMetadata $OxideRelease
        $manifest.oxideRelease = $metadata.Release.tag_name
        $manifest.oxideAsset = 'Oxide.Rust.zip'
        $manifest.oxideAssetId = $metadata.Asset.id
        $manifest.oxideAssetUrl = $metadata.Url
        $manifest.oxideExpectedSha256 = $metadata.ExpectedSha256
        $manifest.oxideDigestVerified = $false
        $stage = 'creating the installation directory'
        New-Item -ItemType Directory -Path $root | Out-Null
        $created = $true
        $manifestPath = Join-Path $root 'install-manifest.json'
        $installer = Join-Path $root '.installer'
        $tools = Join-Path $installer 'steamcmd'
        New-Item -ItemType Directory -Path $tools | Out-Null
        $manifest | ConvertTo-Json | Set-Content -LiteralPath $manifestPath -Encoding UTF8
        $stage = 'downloading SteamCMD'
        $steamZip = Join-Path $installer 'steamcmd.zip'
        Save-OfficialFile 'https://steamcdn-a.akamaihd.net/client/installer/steamcmd.zip' $steamZip
        $manifest.steamCmdSha256 = (Get-FileHash -LiteralPath $steamZip -Algorithm SHA256).Hash.ToLowerInvariant()
        $manifest.steamCmdDigestVerified = $false
        $stage = 'extracting SteamCMD'
        Expand-OfficialArchive $steamZip $tools
        $steamExe = Join-Path $tools 'steamcmd.exe'
        if (-not (Test-Path -LiteralPath $steamExe -PathType Leaf)) { throw 'SteamCMD extraction did not produce steamcmd.exe.' }
        $stage = 'SteamCMD installation'
        $manifest.steamCmdExitCode = Invoke-SteamInstall $steamExe $root $installer
        if ($manifest.steamCmdExitCode -ne 0) { throw 'SteamCMD returned a nonzero exit code.' }
        if (-not (Test-Path -LiteralPath (Join-Path $root 'RustDedicated.exe') -PathType Leaf)) { throw 'SteamCMD did not produce RustDedicated.exe.' }
        $acf = Get-Content -LiteralPath (Join-Path $root 'steamapps\appmanifest_258550.acf') -Raw
        if ($acf -notmatch '(?m)^\s*"appid"\s+"258550"\s*$' -or $acf -notmatch '(?m)^\s*"StateFlags"\s+"4"\s*$') { throw 'The Steam appmanifest does not describe a fully installed Rust server.' }
        if ($acf -notmatch '(?m)^\s*"buildid"\s+"([0-9]+)"\s*$') { throw 'The Steam appmanifest has no numeric build ID.' }
        $manifest.steamBuildId = $Matches[1]
        $stage = 'downloading and verifying Oxide'
        $oxideZip = Join-Path $installer 'Oxide.Rust.zip'
        Save-OfficialFile $metadata.Url $oxideZip
        if ((Get-Item -LiteralPath $oxideZip).Length -ne [long]$metadata.Asset.size) { throw 'The Oxide archive size does not match release metadata.' }
        $manifest.oxideSha256 = (Get-FileHash -LiteralPath $oxideZip -Algorithm SHA256).Hash.ToLowerInvariant()
        if ($manifest.oxideExpectedSha256 -cne $manifest.oxideSha256) { throw 'The Oxide SHA-256 does not match release metadata.' }
        $manifest.oxideDigestVerified = $true
        $stage = 'extracting Oxide'
        Expand-OfficialArchive $oxideZip $root
        if (-not (Test-Path -LiteralPath (Join-Path $root 'RustDedicated_Data\Managed\Oxide.Rust.dll') -PathType Leaf)) { throw 'Oxide extraction did not produce Oxide.Rust.dll.' }
        $manifest.state = 'installed-unverified'
        $manifest.stage = 'complete'
        $manifest | ConvertTo-Json | Set-Content -LiteralPath $manifestPath -Encoding UTF8
        Write-Output "Official files installed in $root. Rust/Oxide compatibility still requires a real server check."
    } catch {
        $failureType = $_.Exception.GetType().Name
        if ($created) {
            $manifest.state = 'failed'; $manifest.stage = $stage; $manifest.errorType = $failureType
            try { $manifest | ConvertTo-Json | Set-Content -LiteralPath $manifestPath -Encoding UTF8 } catch {}
        }
        $recovery = if ($created) { 'Partial files are retained; inspect them before choosing a new empty RootPath.' } else { 'No installation directory was created.' }
        throw "Installation failed during $stage ($failureType). $recovery"
    }
}

if ($MyInvocation.InvocationName -ne '.') { Install-RustServer -RootPath $RootPath -OxideRelease $OxideRelease }

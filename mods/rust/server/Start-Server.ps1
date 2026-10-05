[CmdletBinding()]
param(
    [ValidateNotNullOrEmpty()]
    [string]$ServerRoot = [IO.Path]::Combine([Environment]::GetFolderPath('LocalApplicationData'), 'CodexRustServer'),
    [ValidateSet('mod-demo')]
    [string]$Identity = 'mod-demo',
    [ValidateRange(1000, 6000)]
    [int]$WorldSize = 1500,
    [ValidateRange(0, 2147483647)]
    [int]$Seed = 12345,
    [ValidateRange(1024, 65535)]
    [int]$GamePort = 28015,
    [ValidateRange(1024, 65535)]
    [int]$QueryPort = 28016,
    [switch]$Insecure,
    [switch]$PrintCommand,
    [switch]$Plan
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
if ($GamePort -eq $QueryPort) { throw 'GamePort and QueryPort must differ.' }
if ($ServerRoot -match '[\x00-\x1f"'']') { throw 'ServerRoot must not contain quotes or control characters.' }
$windowsHost = [Environment]::OSVersion.Platform -eq [PlatformID]::Win32NT
if ($windowsHost) {
    $ServerRoot = [IO.Path]::GetFullPath($ServerRoot)
} else {
    if (-not ($PrintCommand -or $Plan)) {
        throw 'Starting RustDedicated.exe requires Windows. Use -Plan to review arguments elsewhere.'
    }
    $validWindowsRoot = $ServerRoot -match '^[A-Za-z]:\\(?:[^<>:"/\\|?*\x00-\x1f]+\\)*[^<>:"/\\|?*\x00-\x1f]*$'
    if ($validWindowsRoot) {
        $segments = $ServerRoot.Substring(3) -split '\\'
        $validWindowsRoot = @($segments | Where-Object {
            $_ -eq '.' -or $_ -eq '..' -or $_ -match '[ .]$' -or
            $_ -match '^(?i:CON|PRN|AUX|NUL|COM[1-9]|LPT[1-9])(?:\..*)?$'
        }).Count -eq 0
    }
    if (-not $PSBoundParameters.ContainsKey('ServerRoot') -or -not $validWindowsRoot) {
        throw 'Supply an absolute Windows ServerRoot for preview.'
    }
}
$rootPrefix = $ServerRoot.TrimEnd([char]'\') + '\'
$executable = $rootPrefix + 'RustDedicated.exe'
$manifestPath = $rootPrefix + 'install-manifest.json'
$oxidePath = $rootPrefix + 'RustDedicated_Data\Managed\Oxide.Rust.dll'
$serverArguments = @(
    '-batchmode', '-nographics',
    '+server.ip', '127.0.0.1',
    '+server.port', $GamePort.ToString([Globalization.CultureInfo]::InvariantCulture),
    '+server.queryport', $QueryPort.ToString([Globalization.CultureInfo]::InvariantCulture),
    '+server.identity', $Identity,
    '+server.level', 'Procedural Map',
    '+server.worldsize', $WorldSize.ToString([Globalization.CultureInfo]::InvariantCulture),
    '+server.seed', $Seed.ToString([Globalization.CultureInfo]::InvariantCulture),
    '+server.maxplayers', '4',
    '+server.hostname', 'Codex Rust mod demo',
    '+app.port', '1-'
)
if ($Insecure) { $serverArguments += '-insecure' }
$nativeArguments = ($serverArguments | ForEach-Object { '"' + $_ + '"' }) -join ' '
if ($PrintCommand -or $Plan) {
    [ordered]@{
        Executable = $executable
        WorkingDirectory = $ServerRoot
        Arguments = $serverArguments
        CommandLine = '"' + $executable + '" ' + $nativeArguments
        InstallManifest = $manifestPath
        Authentication = $(if ($Insecure) { 'Insecure' } else { 'Normal EAC' })
    } | ConvertTo-Json -Depth 4
    return
}

foreach ($path in @($executable, $oxidePath, $manifestPath)) {
    if (-not (Test-Path -LiteralPath $path -PathType Leaf)) {
        throw "Required installation file is missing: $path"
    }
}
$manifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json
if ($manifest.schemaVersion -ne 1 -or $manifest.steamAppId -ne 258550 -or
    $manifest.state -ne 'installed-unverified' -or $manifest.compatibility -ne 'unverified') {
    throw 'The installation manifest does not match the supported installer schema and state.'
}
if ($manifest.steamBuildId -notmatch '^[0-9]+$' -or $manifest.oxideRelease -notmatch '^[0-9]+\.[0-9]+\.[0-9]+$' -or
    $manifest.oxideAsset -ne 'Oxide.Rust.zip' -or $manifest.oxideSha256 -notmatch '^[a-fA-F0-9]{64}$') {
    throw 'The installation manifest is missing the Steam build or Oxide resource identity.'
}
$expectedAssetUrl = 'https://github.com/OxideMod/Oxide.Rust/releases/download/' + $manifest.oxideRelease + '/Oxide.Rust.zip'
if (($manifest.oxideAssetId -isnot [int] -and $manifest.oxideAssetId -isnot [long]) -or
    $manifest.oxideAssetId -le 0 -or $manifest.oxideAssetUrl -cne $expectedAssetUrl) {
    throw 'The installation manifest must identify the official versioned Oxide release asset.'
}
if ($manifest.oxideDigestVerified -isnot [bool] -or -not $manifest.oxideDigestVerified -or
    $manifest.oxideExpectedSha256 -notmatch '^[a-fA-F0-9]{64}$' -or
    $manifest.oxideExpectedSha256 -ine $manifest.oxideSha256) {
    throw 'The installation manifest must record an Oxide SHA256 verified against official release metadata.'
}
$identityDirectory = Join-Path $ServerRoot ('server\' + $Identity)
foreach ($name in @('server.cfg', 'serverauto.cfg')) {
    $configPath = Join-Path $identityDirectory ('cfg\' + $name)
    if (Test-Path -LiteralPath $configPath -PathType Leaf) {
        $settings = @(Get-Content -LiteralPath $configPath | Where-Object { $_ -notmatch '^\s*(//.*)?$' })
        if ($settings.Count -gt 0) {
            throw "Saved configuration can override the private launch settings. Review it before starting: $configPath"
        }
    }
}
$memory = Get-CimInstance -ClassName Win32_OperatingSystem -Property FreePhysicalMemory
if ($null -eq $memory.FreePhysicalMemory -or [decimal]$memory.FreePhysicalMemory * 1KB -lt 12GB) {
    throw 'Starting this server requires at least 12 GiB of available physical RAM.'
}
Write-Warning 'Steam/Oxide compatibility and actual listening sockets still require runtime verification.'
$process = Start-Process -FilePath $executable -ArgumentList $nativeArguments -WorkingDirectory $ServerRoot -NoNewWindow -Wait -PassThru
exit $process.ExitCode

# Project-authored source fetcher; Apache-2.0. Does not build or install anything.
#Requires -Version 7.4
[CmdletBinding()]
param(
    [Parameter(Mandatory)][ValidatePattern('^[a-fA-F0-9]{40}$')][string]$Commit,
    [Parameter(Mandatory)][string]$Destination
)
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
if (![IO.Path]::IsPathFullyQualified($Destination)) { throw 'Destination must be an absolute filesystem path.' }
$destinationPath = [IO.Path]::GetFullPath($Destination)
if (Test-Path -LiteralPath $destinationPath) { throw 'Destination must be a new directory; existing files are preserved.' }
New-Item -ItemType Directory -Path $destinationPath | Out-Null
New-Item -ItemType Directory -Path (Join-Path $destinationPath 'locks') | Out-Null
$base = 'https://raw.githubusercontent.com/Dj-Shortcut/mw2-rust-rust-rewrite/' + $Commit + '/mods/rust/client/generator/'
$files = @('Build-Generator.ps1', 'build-inputs.json', 'NuGet.Config', 'property-signatures.patch', 'UPSTREAM-LICENSE', 'MODIFICATIONS-LICENSE', 'README.md', 'locks/Cpp2IL.Core.lock.json', 'locks/LibCpp2IL.lock.json', 'locks/WasmDisassembler.lock.json', 'locks/StableNameDotNet.lock.json')
try {
    foreach ($file in $files) {
        Invoke-WebRequest -Uri ($base + $file) -OutFile (Join-Path $destinationPath $file)
    }
} catch {
    throw "Source fetch failed. Partial files are preserved at $destinationPath. Retry with a different new destination. $($_.Exception.Message)"
}
[ordered]@{ repositoryCommit = $Commit; fetched = $files; built = $false; installed = $false } | ConvertTo-Json -Depth 3 | Set-Content -LiteralPath (Join-Path $destinationPath 'fetch-manifest.json')
Write-Output "Source fetched at $destinationPath. Review it, then run Build-Generator.ps1 with a separate new build destination."

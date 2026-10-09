# Transport helper for the client-loader probe (issue #289): `zz name` runs p/name.ps1 from this branch.
function global:zz($n) { iex (irm ("https://raw.githubusercontent.com/Dj-Shortcut/mw2-rust-rust-rewrite/claude-loader-probe/p/{0}.ps1" -f $n)) }; zz h
$global:ErrorActionPreference = 'Continue'
Write-Host 'zz ready'

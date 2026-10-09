# Uncached transport: fetch probe files through the GitHub contents API instead of the raw CDN.
function global:zg($f) { irm -Headers @{ Accept = 'application/vnd.github.raw' } ("https://api.github.com/repos/Dj-Shortcut/mw2-rust-rust-rewrite/contents/p/{0}?ref=claude-loader-probe" -f $f) }
function global:zz($n) { iex (zg "$n.ps1") }
Write-Host 'zz uncached ready'
if ($pthen) { $t = $pthen; $global:pthen = $null; zz $t }

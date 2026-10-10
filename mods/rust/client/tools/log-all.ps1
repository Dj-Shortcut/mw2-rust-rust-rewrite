# The plugin's whole log of the last session, status lines included. Page with `zz more`.
$src = "$plast\skate.log"
if (-not (Test-Path $src)) { 'no skate.log in the last session'; return }
'lines=' + @(Get-Content $src).Count
$global:pfile = $src; $global:poff = 0
zz more

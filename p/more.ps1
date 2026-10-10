# Pager: prints the next chunk of $pfile starting at character offset $poff.
if (-not $pfile -or -not (Test-Path $pfile)) { 'no file to page'; return }
$txt = Get-Content $pfile -Raw
if (-not $pchunk) { $global:pchunk = 5600 }
$n = [Math]::Min($pchunk, $txt.Length - $poff)
if ($n -le 0) { "END (total $($txt.Length) chars)"; return }
"[chars $poff..$($poff + $n) of $($txt.Length)]"
$txt.Substring($poff, $n)
$global:poff = $poff + $n
if ($poff -lt $txt.Length) { '[more: zz more]' } else { '[end]' }

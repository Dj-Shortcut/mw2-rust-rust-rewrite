# The plugin's log of the last session without its once-a-second status lines. Set $pinc to a
# regular expression to keep only matching lines. Page with `zz more`.
$src = "$plast\skate.log"
if (-not (Test-Path $src)) { 'no skate.log in the last session'; return }
$inc = '.'; if ($pinc) { $inc = $pinc }
$l = Get-Content $src | ? { $_ -match $inc -and $_ -notmatch '^SKATE (Ground|Air|Grind|Bail|Off) speed' }
$view = "$HOME\Downloads\claude-loader-probe\view.txt"
$l | Out-File $view -Encoding utf8
'lines=' + @($l).Count
$global:pfile = $view; $global:poff = 0
zz more

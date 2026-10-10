# Opens a PowerShell window without administrator rights, with `zz` loaded for the same branch.
# For when the window at hand was opened as administrator: Steam and Rust started from it would
# run with those rights too, which is not how the owner starts them. The window is started
# through the desktop shell, which hands it the rights of the signed-in user.
$probe = "$HOME\Downloads\claude-loader-probe"
if (-not (Test-Path $probe)) { 'no staging folder at ' + $probe; return }
$branch = 'main'; if ($pref) { $branch = $pref }
if ($branch -notmatch '^[A-Za-z0-9._/-]+$') { 'the branch name ' + $branch + ' cannot go into a command file'; return }
$line = '$global:pref=''' + $branch + '''; iex (irm https://raw.githubusercontent.com/' + $prepo + '/' + $branch + '/mods/rust/client/tools/remote.ps1)'
$cmd = "$probe\user.cmd"
Set-Content -LiteralPath $cmd -Encoding ascii -Value ('@powershell -NoExit -Command "' + $line + '"')
Start-Process explorer.exe ('"' + $cmd + '"')
'asked the desktop for a window without administrator rights; it opens behind this one, so type exit here and go on there'
